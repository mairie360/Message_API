use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{require_chat_access, AccessDenied};

use crate::database::chats::delete_empty_chat::view::DeleteEmptyChatQueryView;
use crate::database::chats::remove_user_from_chat::view::RemoveMemberFromChatQueryView;
use crate::endpoints::v1::id::users::id::UsersPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveUserFromChatError {
    DatabaseError,
    BadRequest,
    NotFound,
}

impl std::fmt::Display for RemoveUserFromChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoveUserFromChatError::NotFound => write!(f, "Unknown chat."),
            RemoveUserFromChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            RemoveUserFromChatError::BadRequest => {
                write!(f, "Bad request.")
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
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_remove_user_from_chat(
    state: web::Data<AppState>,
    chat_id: u64,
    user_id: u64,
) -> Result<(), RemoveUserFromChatError> {
    let view = RemoveMemberFromChatQueryView::new(chat_id, user_id);
    state
        .get_smart_db()
        .fetch_scalar::<i32, _>(&view)
        .await
        .map_err(|e| match e {
            ApiLibError::Database(DbError::NotFound) => RemoveUserFromChatError::BadRequest,
            _ => RemoveUserFromChatError::DatabaseError,
        })?;

    // A chat lives as long as it has members: the last one leaving deletes it.
    state
        .get_smart_db()
        .execute(DeleteEmptyChatQueryView::new(chat_id))
        .await
        .map_err(|_| RemoveUserFromChatError::DatabaseError)?;
    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Remove a member from a chat",
    description = "Removes a user from a chat, which leaves their `GET /api/v1/`; their messages are kept. Any \
                   member may remove any member, themselves included (leaving the chat).\n\n \
                   **The chat is deleted, with its messages, when its last member is removed.**\n\n \
                   Removing a user who is not a member answers `400`.\n\nOnly the members of the chat may call this route; administrators bypass the check. A caller who is not a member gets the same `404` as for an unknown chat.",
    responses(
        (
            status = 204,
            description = "Participant retiré, ou déjà absent. Corps vide.",
        ),
        (
            status = 400,
            description = "Un segment de l'URL n'est pas un entier, ou le corps JSON est malformé.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
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
            description = "Erreur de base de données.",
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
    require_chat_access(&state, chat_id, auth_user.id).await?;
    trigger_remove_user_from_chat(state, chat_id, user_id).await?;
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
