use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{
    begin, commit, require_chat_access_in, AccessDenied, DIRECT_CHAT_MEMBERS_MESSAGE,
    NOT_A_MANAGER_MESSAGE,
};

use crate::database::chats::delete_empty_chat::view::DeleteEmptyChatQueryView;
use crate::database::chats::hide_direct_chat::view::HideDirectChatQueryView;
use crate::database::chats::remove_user_from_chat::view::RemoveMemberFromChatQueryView;
use crate::endpoints::error::{classify, unexpected, DbFailure};
use crate::endpoints::v1::id::users::id::UsersPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveUserFromChatError {
    DatabaseError,
    BadRequest,
    NotFound,
    Forbidden,
    DirectChat,
}

impl std::fmt::Display for RemoveUserFromChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoveUserFromChatError::NotFound => write!(f, "Unknown chat."),
            RemoveUserFromChatError::Forbidden => f.write_str(NOT_A_MANAGER_MESSAGE),
            RemoveUserFromChatError::DirectChat => f.write_str(DIRECT_CHAT_MEMBERS_MESSAGE),
            RemoveUserFromChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            RemoveUserFromChatError::BadRequest => {
                write!(f, "This user is not a member of the chat.")
            }
        }
    }
}

impl ResponseError for RemoveUserFromChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            RemoveUserFromChatError::NotFound => StatusCode::NOT_FOUND,
            RemoveUserFromChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            RemoveUserFromChatError::BadRequest => StatusCode::BAD_REQUEST,
            RemoveUserFromChatError::Forbidden | RemoveUserFromChatError::DirectChat => {
                StatusCode::FORBIDDEN
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_remove_user_from_chat(
    tx: &mut SmartTransaction,
    chat_id: u64,
    user_id: u64,
    is_direct: bool,
) -> Result<(), RemoveUserFromChatError> {
    // A participant of a direct chat only hides it: they stay its participant, and the next
    // message shows it to them again (MAIR-478).
    let removed = if is_direct {
        tx.fetch_scalar::<i32, _>(&HideDirectChatQueryView::new(chat_id, user_id))
            .await
    } else {
        tx.fetch_scalar::<i32, _>(&RemoveMemberFromChatQueryView::new(chat_id, user_id))
            .await
    };
    removed.map_err(|e| match classify("remove chat member", e) {
        DbFailure::NotFound => RemoveUserFromChatError::BadRequest,
        _ => RemoveUserFromChatError::DatabaseError,
    })?;

    // A chat lives as long as someone sees it: the last member leaving (or the second participant
    // hiding a direct chat) deletes it, in the same transaction, so a failure never leaves a chat
    // nobody sees.
    tx.execute(&DeleteEmptyChatQueryView::new(chat_id))
        .await
        .map_err(|e| {
            unexpected("delete empty chat", e);
            RemoveUserFromChatError::DatabaseError
        })?;
    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Remove a member from a chat",
    description = "Removes a user from a chat, which leaves their `GET /api/v1/`; their messages are kept.\n\n \
                   **Who may remove whom:** any member may remove **themselves** (leave the chat). Removing \
                   **someone else** is reserved to the creator of the chat (while still a member) and to \
                   administrators; any other member gets `403`.\n\n \
                   **The chat is deleted, with its messages, when its last member is removed.**\n\n \
                   **Direct chat:** a participant may only remove themselves, which **hides** the chat: it leaves \
                   their `GET /api/v1/`, they stay its participant (`contact_id` on the other side) and the next \
                   message posted in it, by either participant, shows it to them again. Removing the other \
                   participant answers `403`, administrators included. The chat is deleted once both have hidden it.\n\n \
                   Removing a user who is not a member answers `400`.\n\n \
                   A caller who is neither a member nor an administrator gets the same `404` as for an unknown chat.",
    responses(
        (
            status = 204,
            description = "Member removed. Empty body.",
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, or `user_id` is not a member of the chat (for a direct chat: has already hidden it).",
            body = String,
            content_type = "text/plain",
            example = json!("This user is not a member of the chat.")
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
            description = "The caller removes someone else from a group chat but is neither its creator (still a member) nor an administrator (`Only the creator of the chat or an administrator can manage its other members.`), or removes the other participant of a direct chat (`A direct chat always keeps its two participants: create a group chat instead.`).",
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
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    params(
        UsersPathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[delete("/")]
pub async fn remove_user_from_chat(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<UsersPathParams>,
) -> Result<impl Responder, RemoveUserFromChatError> {
    let chat_id = params.chat_id();
    let user_id = params.user_id();
    let mut tx = begin(&state).await?;
    let role = require_chat_access_in(&mut tx, chat_id, auth_user.id).await?;
    if user_id != auth_user.id {
        if role.is_direct {
            return Err(RemoveUserFromChatError::DirectChat);
        }
        if !role.can_manage_members() {
            return Err(RemoveUserFromChatError::Forbidden);
        }
    }
    trigger_remove_user_from_chat(&mut tx, chat_id, user_id, role.is_direct).await?;
    commit(tx).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for RemoveUserFromChatError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => RemoveUserFromChatError::NotFound,
            AccessDenied::DatabaseError => RemoveUserFromChatError::DatabaseError,
        }
    }
}
