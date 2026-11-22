use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{
    begin, commit, require_chat_access_in, AccessDenied, DIRECT_CHAT_MEMBERS_MESSAGE,
    NOT_A_MANAGER_MESSAGE,
};

use crate::database::chats::add_users_to_chat::view::AddMembersToChatQueryView;
use crate::database::chats::system_notes::view::SystemAction;
use crate::endpoints::error::{classify, DbFailure};
use crate::endpoints::v1::id::users::post::view::{AddUsersToChat, AddUsersToChatResultView};
use crate::endpoints::v1::id::users::{post_system_note, SystemNoteFailed};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedJson;
use crate::sse::state::ChatEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum AddUsersToChatError {
    DatabaseError,
    UnknownChat,
    UnknownUser,
    AlreadyMember,
    DirectChat,
    Forbidden,
}

impl std::fmt::Display for AddUsersToChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddUsersToChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            AddUsersToChatError::UnknownChat => write!(f, "Unknown chat."),
            AddUsersToChatError::Forbidden => f.write_str(NOT_A_MANAGER_MESSAGE),
            AddUsersToChatError::UnknownUser => write!(f, "`users_id` contains an unknown user."),
            AddUsersToChatError::AlreadyMember => {
                write!(f, "A user of `users_id` is already a member of this chat.")
            }
            AddUsersToChatError::DirectChat => f.write_str(DIRECT_CHAT_MEMBERS_MESSAGE),
        }
    }
}

impl ResponseError for AddUsersToChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddUsersToChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AddUsersToChatError::UnknownChat => StatusCode::NOT_FOUND,
            AddUsersToChatError::UnknownUser => StatusCode::BAD_REQUEST,
            AddUsersToChatError::AlreadyMember | AddUsersToChatError::DirectChat => {
                StatusCode::CONFLICT
            }
            AddUsersToChatError::Forbidden => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_add_users_to_chat(
    tx: &mut SmartTransaction,
    chat_id: u64,
    actor_id: u64,
    view: AddUsersToChat,
) -> Result<AddUsersToChatResultView, AddUsersToChatError> {
    let added = view.users_id().to_vec();
    let view = AddMembersToChatQueryView::new(chat_id, added.clone());
    if view.is_empty() {
        return Ok(AddUsersToChatResultView::new(chat_id, Vec::new()));
    }

    tx.execute(&view)
        .await
        .map_err(|e| match classify("add chat members", e) {
            // Both foreign keys of conversation_members: tell the missing chat from the missing user.
            DbFailure::ForeignKey(message) if message.contains("conversation_id") => {
                AddUsersToChatError::UnknownChat
            }
            DbFailure::ForeignKey(_) => AddUsersToChatError::UnknownUser,
            DbFailure::Unique(_) => AddUsersToChatError::AlreadyMember,
            _ => AddUsersToChatError::DatabaseError,
        })?;

    // The history records the addition (MAIR-1155), in the same transaction.
    let note = post_system_note(tx, chat_id, actor_id, SystemAction::MembersAdded, &added).await?;
    Ok(AddUsersToChatResultView::new(chat_id, added).with_system_message(note))
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    summary = "Add members to a chat",
    description = "Adds one or more users to a chat in a single call; the chat, **with its whole history**, \
                   then shows up in their `GET /api/v1/`. The call is all or nothing: one unknown user or one \
                   user already member and nobody is added.\n\n \
                   **Only the creator of the chat (while still a member) and administrators** may add users; \
                   any other member gets `403`. A caller who is neither a member nor an administrator gets \
                   the same `404` as for an unknown chat.\n\n \
                   A **direct** chat keeps its two participants: adding anyone to it answers `409`, create a \
                   group chat with `POST /api/v1/` instead.\n\n \
                   **System note** (MAIR-1155): the addition is recorded in the history as a message of `kind` \
                   `system` (action `members_added`, the caller as `actor_id`, the users added as `user_ids`), \
                   returned as `system_message`. Once committed, it is signalled as `NEW_MSG` to the members but \
                   the caller, and `MEMBERS_CHANGED` goes to every member, the caller and the new ones included.",
    responses(
        (
            status = 200,
            description = "Users added. `added` lists them (every id of `users_id`); `system_message` is the note the history now shows (MAIR-1155).",
            body = AddUsersToChatResultView,
            example = json!({ "chat_id": 5, "added": [42, 51], "system_message": {
                "id": 119, "content": "", "sender_id": 1, "created_at": "2026-09-16T09:20:00Z",
                "citation": null, "quoted": null, "edited_at": null,
                "updated_at": "2026-09-16T09:20:00.120411Z", "deleted": false,
                "kind": "system",
                "system_event": { "action": "members_added", "actor_id": 1, "user_ids": [42, 51] },
                "mentions": [], "mentions_everyone": false, "group_mentions": [], "links": [], "reactions": []
            } })
        ),
        (
            status = 400,
            description = "Malformed JSON body, `chat_id` not an integer, `users_id` missing, empty, longer than 50, with a duplicate or an id outside 1..=2147483647 (`Invalid `users_id`: …`), or containing an unknown user.",
            body = String,
            content_type = "text/plain",
            example = json!("`users_id` contains an unknown user.")
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
            description = "The caller is a member but neither the creator of the chat nor an administrator.",
            body = String,
            content_type = "text/plain",
            example = json!("Only the creator of the chat or an administrator can manage its other members.")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`, or the caller is neither one of its members nor an administrator (both cases are deliberately indistinguishable).",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown chat.")
        ),
        (
            status = 409,
            description = "A user of `users_id` is already a member of the chat (`A user of `users_id` is already a member of this chat.`), or the chat is a direct chat (`A direct chat always keeps its two participants: create a group chat instead.`); nobody is added.",
            body = String,
            content_type = "text/plain",
            example = json!("A user of `users_id` is already a member of this chat.")
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
        content = AddUsersToChat,
        description = "Core API ids of the users to attach.",
        example = json!({ "users_id": [42, 51] })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[post("/")]
pub async fn add_users_to_chat(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::sse::state::AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<AddUsersToChat>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, AddUsersToChatError> {
    let view = view.into_inner();
    let mut tx = begin(&state).await?;
    let role = require_chat_access_in(&mut tx, params.chat_id, auth_user.id).await?;
    if role.is_direct {
        return Err(AddUsersToChatError::DirectChat);
    }
    if !role.can_manage_members() {
        return Err(AddUsersToChatError::Forbidden);
    }
    let result = trigger_add_users_to_chat(&mut tx, params.chat_id, auth_user.id, view).await?;
    commit(tx).await?;
    // Only once committed: the new members can read the chat when they get the signals.
    if let Some(note) = result.system_message() {
        sse_state
            .publish(ChatEvent::message_created(
                params.chat_id,
                auth_user.id,
                note.id(),
            ))
            .await;
    }
    sse_state
        .publish(ChatEvent::members_changed(
            params.chat_id,
            auth_user.id,
            Vec::new(),
        ))
        .await;
    Ok(HttpResponse::Ok().json(result))
}

impl From<SystemNoteFailed> for AddUsersToChatError {
    fn from(_: SystemNoteFailed) -> Self {
        AddUsersToChatError::DatabaseError
    }
}

impl From<AccessDenied> for AddUsersToChatError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => AddUsersToChatError::UnknownChat,
            AccessDenied::DatabaseError => AddUsersToChatError::DatabaseError,
        }
    }
}
