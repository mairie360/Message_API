use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{begin, chat_access_in, commit, AccessDenied};

use crate::database::chats::access::view::ChatAccess;
use crate::database::chats::delete_chat::view::DeleteChatQueryView;
use crate::database::chats::delete_right::view::ChatDeleteRightQueryView;
use crate::endpoints::error::{classify, unexpected, DbFailure};
use crate::endpoints::v1::id::ChatPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteChatError {
    DatabaseError,
    NothingToDelete,
    UnknownChat,
    Forbidden,
}

impl std::fmt::Display for DeleteChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteChatError::Forbidden => write!(
                f,
                "Only the creator of the chat, an administrator or an agent granted the right can delete it."
            ),
            DeleteChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteChatError::NothingToDelete => {
                write!(f, "Nothing to delete.")
            }
            DeleteChatError::UnknownChat => {
                write!(f, "Unknown chat.")
            }
        }
    }
}

impl ResponseError for DeleteChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteChatError::Forbidden => StatusCode::FORBIDDEN,
            DeleteChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteChatError::NothingToDelete => StatusCode::OK,
            DeleteChatError::UnknownChat => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// Whether `user_id` may delete the chat: an administrator, the creator of a group chat while
/// still a member, or anyone `check_access` grants `delete` on the conversation (global
/// `delete_all`, individual or group ACL), member or not.
async fn may_delete(
    tx: &mut SmartTransaction,
    access: ChatAccess,
    chat_id: u64,
    user_id: u64,
) -> Result<bool, DeleteChatError> {
    if access.is_admin || (!access.is_direct && access.is_member && access.is_creator) {
        return Ok(true);
    }
    tx.fetch_scalar::<bool, _>(&ChatDeleteRightQueryView::new(chat_id, user_id))
        .await
        .map_err(|e| {
            unexpected("chat delete right", e);
            DeleteChatError::DatabaseError
        })
}

async fn trigger_delete_chat(
    tx: &mut SmartTransaction,
    chat_id: u64,
    performed_by: u64,
) -> Result<(), DeleteChatError> {
    // Logged in messaging_moderation_log by the same statement.
    let view = DeleteChatQueryView::new(chat_id, performed_by);
    tx.fetch_scalar::<i32, _>(&view)
        .await
        .map_err(|e| match classify("delete chat", e) {
            DbFailure::NotFound => DeleteChatError::UnknownChat,
            _ => DeleteChatError::DatabaseError,
        })?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Delete a chat",
    description = "Permanently deletes a chat, its messages and its members, for everyone. The deletion is recorded, \
                   with the title of the chat and who deleted it, in the moderation log.\n\n \
                   **Who may delete:**\n\
                   - a **group** chat: its creator (while still a member), an administrator, or an agent the \
                   platform grants the `delete` right on this conversation (`check_access`: global `delete_all` \
                   permission, individual or group ACL), member or not;\n\
                   - a **direct** chat: an administrator or an agent granted the right (moderation). Its \
                   participants hide it with `DELETE /api/v1/{chat_id}/users/{own id}/` instead; it is deleted \
                   once both have.\n\n \
                   Any chat is also deleted with its last member. A member who may not delete it gets `403`; anyone \
                   else gets the same `404` as for an unknown chat.",
    responses(
        (
            status = 204,
            description = "Chat deleted. Empty body.",
        ),
        (
            status = 400,
            description = "A URL segment is not an integer.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
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
            description = "The caller is a member of the chat but neither the creator of a group chat, nor an administrator, nor granted the `delete` right on it.",
            body = String,
            content_type = "text/plain",
            example = json!("Only the creator of the chat, an administrator or an agent granted the right can delete it.")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`, or the caller is not one of its members and may not delete it (both cases are deliberately indistinguishable).",
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
        ChatPathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[delete("/")]
pub async fn delete_chat(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, DeleteChatError> {
    let chat_id = params.chat_id;
    let mut tx = begin(&state).await?;
    let access = chat_access_in(&mut tx, chat_id, auth_user.id).await?;
    if !access.chat_exists {
        return Err(DeleteChatError::UnknownChat);
    }
    if !may_delete(&mut tx, access, chat_id, auth_user.id).await? {
        // Only a member learns that the chat exists.
        return Err(if access.is_member {
            DeleteChatError::Forbidden
        } else {
            DeleteChatError::UnknownChat
        });
    }
    trigger_delete_chat(&mut tx, chat_id, auth_user.id).await?;
    commit(tx).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for DeleteChatError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => DeleteChatError::UnknownChat,
            AccessDenied::DatabaseError => DeleteChatError::DatabaseError,
        }
    }
}
