use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{
    require_chat_access, require_message_author, AccessDenied, MessageDenied,
};

use crate::database::chats::delete_message_from_chat::view::DeleteMessageQueryView;
use crate::endpoints::v1::id::messages::id::MessagePathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteMessageError {
    DatabaseError,
    UnknownMessage,
    Forbidden,
}

impl std::fmt::Display for DeleteMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteMessageError::Forbidden => {
                write!(f, "Only the author of a message can delete it.")
            }
            DeleteMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteMessageError::UnknownMessage => {
                write!(f, "Unknown message.")
            }
        }
    }
}

impl ResponseError for DeleteMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteMessageError::Forbidden => StatusCode::FORBIDDEN,
            DeleteMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteMessageError::UnknownMessage => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_message(
    state: web::Data<AppState>,
    message_id: u64,
) -> Result<(), DeleteMessageError> {
    let view = DeleteMessageQueryView::new(message_id);
    state
        .get_smart_db()
        .fetch_scalar::<i64, _>(&view)
        .await
        .map_err(|e| match e {
            ApiLibError::Database(DbError::NotFound) => DeleteMessageError::UnknownMessage,
            _ => DeleteMessageError::DatabaseError,
        })?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Delete a message",
    description = "Permanently deletes a message of a chat.\n\n \
                   No `ChatSignal` is pushed on the SSE stream: the other members still see the message until they \
                   reload the chat.\n\n \
                   Only the author of the message may delete it (`403` for another member); administrators bypass \
                   the check.",
    responses(
        (
            status = 204,
            description = "Message supprimé. Corps vide.",
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
            status = 403,
            description = "The message was written by another member.",
            body = String,
            content_type = "text/plain",
            example = json!("Only the author of a message can delete it.")
        ),
        (
            status = 404,
            description = "No message `message_id` in this chat, or the caller is neither a member of the chat nor an administrator.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown message.")
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
        MessagePathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[delete("/")]
pub async fn delete_message(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
) -> Result<impl Responder, DeleteMessageError> {
    let message_id = params.message_id();
    let is_admin = require_chat_access(&state, params.chat_id(), auth_user.id).await?;
    require_message_author(&state, params.chat_id(), message_id, auth_user.id, is_admin).await?;
    trigger_delete_message(state, message_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for DeleteMessageError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => DeleteMessageError::UnknownMessage,
            AccessDenied::DatabaseError => DeleteMessageError::DatabaseError,
        }
    }
}

impl From<MessageDenied> for DeleteMessageError {
    fn from(denied: MessageDenied) -> Self {
        match denied {
            MessageDenied::NotFound => DeleteMessageError::UnknownMessage,
            MessageDenied::Forbidden => DeleteMessageError::Forbidden,
            MessageDenied::DatabaseError => DeleteMessageError::DatabaseError,
        }
    }
}
