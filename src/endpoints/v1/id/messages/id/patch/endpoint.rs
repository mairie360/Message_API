use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::sse::state::ChatEvent;

use crate::endpoints::v1::id::access::{
    begin, commit, require_chat_access_in, require_message_author_in, AccessDenied, MessageDenied,
};

use crate::database::chats::get_chat::view::{GetMessageQueryView, Message};
use crate::database::chats::message_reactions::view::{MessageState, MessageStateQueryView};
use crate::database::chats::patch_message_in_chat::view::PatchMessageQueryView;
use crate::endpoints::error::{classify, unexpected, DbFailure};
use crate::endpoints::v1::id::get::view::MessageView;
use crate::endpoints::v1::id::messages::id::patch::view::PatchMessageView;
use crate::endpoints::v1::id::messages::id::MessagePathParams;
use crate::endpoints::v1::id::messages::tags::{
    check_tags, mention_recipients, write_tags, TagsFailure,
};
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchMessageError {
    DatabaseError,
    UnknownMessage,
    UnknownChat,
    Forbidden,
    /// The message is a tombstone (MAIR-524): it is never edited back.
    DeletedMessage,
    /// The message is a system note (MAIR-1155): nobody edits it.
    SystemNote,
    /// A mention or link the author may not make (MAIR-1155): the body of the `400`.
    RefusedTag(&'static str),
}

/// Body of the `400` answered to a change of a system note.
pub const SYSTEM_NOTE_MESSAGE: &str = "A system note cannot be changed.";

impl std::fmt::Display for PatchMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchMessageError::Forbidden => write!(f, "Only the author of a message can edit it."),
            PatchMessageError::UnknownChat => write!(f, "Unknown chat."),
            PatchMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PatchMessageError::UnknownMessage => write!(f, "Unknown message."),
            PatchMessageError::DeletedMessage => write!(f, "Unknown message."),
            PatchMessageError::SystemNote => f.write_str(SYSTEM_NOTE_MESSAGE),
            PatchMessageError::RefusedTag(message) => f.write_str(message),
        }
    }
}

impl ResponseError for PatchMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            PatchMessageError::Forbidden => StatusCode::FORBIDDEN,
            // An unknown message is a `404`, as for `DELETE` and the reactions of the same message.
            PatchMessageError::UnknownChat
            | PatchMessageError::UnknownMessage
            | PatchMessageError::DeletedMessage => StatusCode::NOT_FOUND,
            PatchMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PatchMessageError::SystemNote | PatchMessageError::RefusedTag(_) => {
                StatusCode::BAD_REQUEST
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// The message edited and the members it newly mentions.
struct Patched {
    message: MessageView,
    newly_mentioned: Vec<u64>,
}

async fn trigger_patch_message(
    tx: &mut SmartTransaction,
    chat_id: u64,
    message_id: u64,
    author_id: u64,
    view: PatchMessageView,
) -> Result<Patched, PatchMessageError> {
    let state: MessageState = tx
        .fetch_one(&MessageStateQueryView::new(chat_id, message_id))
        .await
        .map_err(|e| match classify("message state", e) {
            DbFailure::NotFound => PatchMessageError::UnknownMessage,
            _ => PatchMessageError::DatabaseError,
        })?;
    if state.deleted {
        return Err(PatchMessageError::DeletedMessage);
    }
    if state.is_system() {
        return Err(PatchMessageError::SystemNote);
    }
    let tags = view.tags();
    check_tags(tx, chat_id, author_id, &tags).await?;
    let changes_mentions = tags.has_lists() || tags.mentions_everyone.is_some();
    let mentioned_before = if changes_mentions {
        mention_recipients(tx, message_id).await?
    } else {
        Vec::new()
    };

    let update = PatchMessageQueryView::new(chat_id, message_id, view.content())
        .mentioning_everyone(tags.mentions_everyone);
    tx.fetch_scalar::<i64, _>(&update)
        .await
        .map_err(|e| match classify("patch message", e) {
            // The state check found a live message: no row left means it was deleted meanwhile.
            DbFailure::NotFound => PatchMessageError::DeletedMessage,
            _ => PatchMessageError::DatabaseError,
        })?;
    write_tags(tx, message_id, &tags).await?;

    let newly_mentioned = if changes_mentions {
        mention_recipients(tx, message_id)
            .await?
            .into_iter()
            .filter(|id| !mentioned_before.contains(id))
            .collect()
    } else {
        Vec::new()
    };
    let message: Message = tx
        .fetch_one(&GetMessageQueryView::new(chat_id, message_id))
        .await
        .map_err(|e| {
            unexpected("read patched message", e);
            PatchMessageError::DatabaseError
        })?;
    Ok(Patched {
        message: MessageView::from(message),
        newly_mentioned,
    })
}

#[utoipa::path(
    patch,
    path = "",
    summary = "Edit a message",
    description = "Replaces the content of a message. `content` is required: this `PATCH` is not a partial update, \
                   it overwrites the text.\n\n \
                   Once committed, a `MSG_EDITED` `ChatSignal` (with `message_id`) is pushed on the SSE stream of \
                   every other connected member, who reads the new content with `GET /api/v1/{chat_id}/?since=` \
                   (the message comes again with its `edited_at`).\n\n \
                   A deleted message cannot be edited (`404`), nor a system note (`400`).\n\n \
                   **Mentions and links** (MAIR-1155): `mentions`, `mentions_everyone`, `group_mentions` and \
                   `links` follow the rules of `POST /api/v1/{chat_id}/messages/`; each one **absent keeps** what the \
                   message has, each one **present replaces** it (`[]` / `false` clears it). The members the edit \
                   newly mentions get a `MENTIONED` signal.\n\n \
                   **Only the author of the message may edit it**, administrators included: anyone else gets \
                   `403` (an administrator may delete a message, never rewrite it). A caller who is neither a \
                   member of the chat nor an administrator gets the same `404` as for an unknown chat. \
                   The response is the message as edited (same fields as in `GET /api/v1/{chat_id}/`; the body \
                   was empty before MAIR-1155).",
    responses(
        (
            status = 200,
            description = "Message edited: the message as it now reads.",
            body = MessageView,
            example = json!({
                "id": 118,
                "content": "The meeting is finally moved to 4 pm.",
                "sender_id": 42,
                "created_at": "2026-09-16T09:12:00Z",
                "citation": null,
                "quoted": null,
                "edited_at": "2026-09-16T09:14:30.105822Z",
                "updated_at": "2026-09-16T09:14:30.105822Z",
                "deleted": false,
                "kind": "user",
                "system_event": null,
                "mentions": [51],
                "mentions_everyone": false,
                "group_mentions": [],
                "links": [{ "kind": "project", "id": 12 }],
                "reactions": [{ "emoji": "👍", "count": 1, "user_ids": [51] }]
            })
        ),
        (
            status = 400,
            description = "Malformed JSON body, URL segment not an integer, `content` breaking its rules (`content` not blank, at most 5000 characters, no control character other than line breaks and tabs), the message is a system note (`A system note cannot be changed.`), or a tag breaks the rules of `POST /api/v1/{chat_id}/messages/` (more than 50 entries, a duplicate, an id out of range, a user who is not a member, a group no member belongs to, an object the author may not read).",
            body = String,
            content_type = "text/plain",
            example = json!("A system note cannot be changed.")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "The message was written by someone else (administrators included).",
            body = String,
            content_type = "text/plain",
            example = json!("Only the author of a message can edit it.")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`, or the caller is neither one of its members nor an administrator (both cases are deliberately indistinguishable, body `Unknown chat.`); or no live message with this id belongs to the chat: never written, written in another chat, or deleted (body `Unknown message.`).",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown chat.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    params(
        MessagePathParams
    ),
    request_body(
        content = PatchMessageView,
        description = "New content of the message, which replaces the old one entirely.",
        example = json!({ "content": "The meeting is finally moved to 4 pm.", "mentions": [42] })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[patch("/")]
pub async fn patch_message(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::sse::state::AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
    view: ValidatedJson<PatchMessageView>,
) -> Result<impl Responder, PatchMessageError> {
    let message_id = params.message_id();
    let mut tx = begin(&state).await?;
    require_chat_access_in(&mut tx, params.chat_id(), auth_user.id).await?;
    // Nobody rewrites the message of someone else, administrators included.
    require_message_author_in(&mut tx, params.chat_id(), message_id, auth_user.id, false).await?;
    let view = view.into_inner();
    let patched =
        trigger_patch_message(&mut tx, params.chat_id(), message_id, auth_user.id, view).await?;
    commit(tx).await?;
    // Only once committed: the members read the edit as soon as they get the signal.
    sse_state
        .publish(ChatEvent::message_edited(
            params.chat_id(),
            auth_user.id,
            message_id,
        ))
        .await;
    if !patched.newly_mentioned.is_empty() {
        sse_state
            .publish(ChatEvent::mentioned(
                params.chat_id(),
                auth_user.id,
                message_id,
                patched.newly_mentioned,
            ))
            .await;
    }
    Ok(HttpResponse::Ok().json(patched.message))
}

impl From<AccessDenied> for PatchMessageError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => PatchMessageError::UnknownChat,
            AccessDenied::DatabaseError => PatchMessageError::DatabaseError,
        }
    }
}

impl From<TagsFailure> for PatchMessageError {
    fn from(failure: TagsFailure) -> Self {
        match failure {
            TagsFailure::Refused(message) => PatchMessageError::RefusedTag(message),
            TagsFailure::DatabaseError => PatchMessageError::DatabaseError,
        }
    }
}

impl From<MessageDenied> for PatchMessageError {
    fn from(denied: MessageDenied) -> Self {
        match denied {
            MessageDenied::NotFound => PatchMessageError::UnknownMessage,
            MessageDenied::Forbidden => PatchMessageError::Forbidden,
            MessageDenied::DatabaseError => PatchMessageError::DatabaseError,
        }
    }
}
