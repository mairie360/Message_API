use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{
    begin, commit, require_chat_access_in, AccessDenied, NotAMember, NOT_A_MEMBER_MESSAGE,
};

use crate::database::chats::add_message_to_chat::view::PostMessageInChatQueryView;
use crate::database::chats::reveal_direct_chat::view::RevealDirectChatQueryView;
use crate::database::ids::bigint_from_sql;
use crate::endpoints::error::{classify, unexpected, DbFailure};
use crate::endpoints::v1::id::messages::post::view::{PostMessageResultView, PostMessageView};
use crate::endpoints::v1::id::messages::tags::{
    check_tags, mention_recipients, write_tags, TagsFailure,
};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedJson;
use crate::realtime::state::ChatEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum PosteMessageError {
    DatabaseError,
    UnknownChat,
    UnknownCitation,
    NotAMember,
    /// A mention or link the author may not make (MAIR-1155): the body of the `400`.
    RefusedTag(&'static str),
}

impl std::fmt::Display for PosteMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PosteMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PosteMessageError::UnknownChat => write!(f, "Unknown chat."),
            PosteMessageError::UnknownCitation => {
                write!(f, "`citation` is not a message of this chat.")
            }
            PosteMessageError::NotAMember => f.write_str(NOT_A_MEMBER_MESSAGE),
            PosteMessageError::RefusedTag(message) => f.write_str(message),
        }
    }
}

impl ResponseError for PosteMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            PosteMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PosteMessageError::UnknownChat => StatusCode::NOT_FOUND,
            PosteMessageError::UnknownCitation | PosteMessageError::RefusedTag(_) => {
                StatusCode::BAD_REQUEST
            }
            PosteMessageError::NotAMember => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// The message posted and the members it mentions.
struct Posted {
    result: PostMessageResultView,
    mentioned: Vec<u64>,
}

async fn trigger_post_message(
    tx: &mut SmartTransaction,
    user_id: u64,
    view: PostMessageView,
    chat_id: u64,
    is_direct: bool,
) -> Result<Posted, PosteMessageError> {
    let tags = view.tags();
    check_tags(tx, chat_id, user_id, &tags).await?;
    if is_direct {
        // A participant who hid the direct chat sees it again, before the insert so that the
        // unread-counter trigger counts this message for them.
        tx.execute(&RevealDirectChatQueryView::new(chat_id))
            .await
            .map_err(|e| {
                unexpected("reveal direct chat", e);
                PosteMessageError::DatabaseError
            })?;
    }
    let view =
        PostMessageInChatQueryView::replying_to(chat_id, user_id, view.content(), view.citation())
            .mentioning_everyone(tags.mentions_everyone.unwrap_or(false));
    let result =
        tx.fetch_scalar::<i64, _>(&view)
            .await
            .map_err(|e| match classify("post message", e) {
                // fk_messages_reply_to: the quoted message is unknown or in another chat.
                DbFailure::ForeignKey(message) if message.contains("reply_to") => {
                    PosteMessageError::UnknownCitation
                }
                DbFailure::ForeignKey(_) => PosteMessageError::UnknownChat,
                // The quoted message is deleted (tombstone): nothing was inserted.
                DbFailure::NotFound => PosteMessageError::UnknownCitation,
                _ => PosteMessageError::DatabaseError,
            })?;

    let message_id = bigint_from_sql(result);
    write_tags(tx, message_id, &tags).await?;
    let mentioned = if tags.has_lists() || tags.mentions_everyone == Some(true) {
        mention_recipients(tx, message_id).await?
    } else {
        Vec::new()
    };
    Ok(Posted {
        result: PostMessageResultView::new(message_id),
        mentioned,
    })
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    summary = "Post a message",
    description = "Adds a message to a chat and pushes a `ChatSignal` on the WebSocket of every other connected \
                   member. The author is taken from the JWT, never from the body.\n\n \
                   `citation` is optional: the id of the message this one answers. It must be a message of the \
                   same chat and not deleted (`400` otherwise) and is read back as `citation` by \
                   `GET /api/v1/{chat_id}/` (`null` once the quoted message is deleted).\n\n \
                   The signal is `NEW_MSG` with the new `message_id`, pushed once the message is committed.\n\n \
                   The response only holds the id given to the message.\n\n \
                   **Mentions and links** (MAIR-1155, optional): `mentions` (members of the chat), \
                   `mentions_everyone`, `group_mentions` (groups one of the members belongs to, see \
                   `GET /api/v1/{chat_id}/mention-groups/`) and `links` (`{kind: project | task | event, id}` the \
                   author may read). They are read back on the message by `GET /api/v1/{chat_id}/`. Every member \
                   mentioned (by id, by `@everyone` or through a group) but the author also gets a `MENTIONED` \
                   signal. A tag the author may not make answers `400` and posts nothing.\n\n \
                   In a **direct** chat, a participant who hid it sees it again (in `GET /api/v1/`, with this \
                   message unread, and on their WebSocket): a direct chat is only hidden until its next message.\n\n \
                   Only the members of the chat may post. An administrator who is not a member gets `403` \
                   (administrators read and moderate, they do not take part); anyone else gets the same `404` \
                   as for an unknown chat.",
    responses(
        (
            status = 200,
            description = "Message posted. The body holds its id.",
            body = PostMessageResultView,
            example = json!({ "id": 101 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, `chat_id` not an integer, `content` breaking its rules (not blank, at most 5000 characters, no control character other than line breaks and tabs), `citation` above 9223372036854775807, `citation` not a message of this chat or a deleted one (`` `citation` is not a message of this chat.``), `mentions` / `group_mentions` / `links` holding more than 50 entries, a duplicate or an id outside 1..=2147483647, a mention of a user who is not a member of the chat, a group no member belongs to, or a link to an object that does not exist or that the author may not read.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `content`: must be at most 5000 characters")
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
            description = "The caller is an administrator who is not a member of the chat.",
            body = String,
            content_type = "text/plain",
            example = json!("Administrators may only read and moderate a chat they are not a member of.")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`, or the caller is neither one of its members nor an administrator (both cases are deliberately indistinguishable).",
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
    request_body(
        content = PostMessageView,
        description = "Content of the message and, optionally, the message it answers.",
        example = json!({ "content": "The meeting is moved to 3 pm.", "citation": 118, "mentions": [42], "mentions_everyone": false, "group_mentions": [], "links": [] })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[post("/")]
pub async fn post_message(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::realtime::state::AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<PostMessageView>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, PosteMessageError> {
    let view = view.into_inner();
    let chat_id = params.chat_id;
    let mut tx = begin(&state).await?;
    let role = require_chat_access_in(&mut tx, chat_id, auth_user.id).await?;
    role.require_member()?;
    let posted = trigger_post_message(&mut tx, auth_user.id, view, chat_id, role.is_direct).await?;
    commit(tx).await?;
    // Only once committed: the members read the new message as soon as they get the signal.
    let message_id = posted.result.id();
    sse_state
        .publish(ChatEvent::message_created(
            chat_id,
            auth_user.id,
            message_id,
        ))
        .await;
    if !posted.mentioned.is_empty() {
        sse_state
            .publish(ChatEvent::mentioned(
                chat_id,
                auth_user.id,
                message_id,
                posted.mentioned,
            ))
            .await;
    }
    Ok(HttpResponse::Ok().json(posted.result))
}

impl From<AccessDenied> for PosteMessageError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => PosteMessageError::UnknownChat,
            AccessDenied::DatabaseError => PosteMessageError::DatabaseError,
        }
    }
}

impl From<TagsFailure> for PosteMessageError {
    fn from(failure: TagsFailure) -> Self {
        match failure {
            TagsFailure::Refused(message) => PosteMessageError::RefusedTag(message),
            TagsFailure::DatabaseError => PosteMessageError::DatabaseError,
        }
    }
}

impl From<NotAMember> for PosteMessageError {
    fn from(_: NotAMember) -> Self {
        PosteMessageError::NotAMember
    }
}
