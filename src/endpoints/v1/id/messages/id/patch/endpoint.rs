use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{
    require_chat_access, require_message_author, AccessDenied, MessageDenied,
};

use crate::database::chats::patch_message_in_chat::view::PatchMessageQueryView;
use crate::endpoints::v1::id::messages::id::patch::view::PatchMessageView;
use crate::endpoints::v1::id::messages::id::MessagePathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchMessageError {
    DatabaseError,
    UnknownEvent,
    UnknownChat,
    Forbidden,
}

impl std::fmt::Display for PatchMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchMessageError::Forbidden => write!(f, "Only the author of a message can edit it."),
            PatchMessageError::UnknownChat => write!(f, "Unknown chat."),
            PatchMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PatchMessageError::UnknownEvent => {
                write!(f, "Unknown event.")
            }
        }
    }
}

impl ResponseError for PatchMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            PatchMessageError::Forbidden => StatusCode::FORBIDDEN,
            PatchMessageError::UnknownChat => StatusCode::NOT_FOUND,
            PatchMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PatchMessageError::UnknownEvent => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_patch_message(
    state: web::Data<AppState>,
    message_id: u64,
    view: PatchMessageView,
) -> Result<(), PatchMessageError> {
    let view = PatchMessageQueryView::new(message_id, view.content());
    state
        .get_smart_db()
        .fetch_scalar::<i64, _>(&view)
        .await
        .map_err(|e| match e {
            ApiLibError::Database(DbError::NotFound) => PatchMessageError::UnknownEvent,
            _ => PatchMessageError::DatabaseError,
        })?;

    Ok(())
}

#[utoipa::path(
    patch,
    path = "",
    summary = "Edit a message",
    description = "Replaces the content of a message. `content` is required: this `PATCH` is not a partial update, \
                   it overwrites the text.\n\n \
                   Unlike posting, no `ChatSignal` is pushed on the SSE stream: the other members only see the edit \
                   when they reload the chat.\n\n \
                   Only the author of the message may edit it (`403` for another member); administrators bypass \
                   the check. A caller who is not a member of the chat gets the same `404` as for an unknown chat. \
                   The response has an empty body.",
    responses(
        (
            status = 200,
            description = "Message modifié. Corps vide.",
        ),
        (
            status = 400,
            description = "Malformed JSON body, URL segment not an integer, `content` breaking its rules (`content` not blank, at most 5000 characters, no `<` / `>`, no control character other than line breaks and tabs), or unknown message (`Unknown event.`).",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown event.")
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
            example = json!("Only the author of a message can edit it.")
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
        MessagePathParams
    ),
    request_body(
        content = PatchMessageView,
        description = "Nouveau contenu du message, qui remplace intégralement l'ancien.",
        example = json!({ "content": "La réunion est finalement décalée à 16h." })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[patch("/")]
pub async fn patch_message(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
    view: ValidatedJson<PatchMessageView>,
) -> Result<impl Responder, PatchMessageError> {
    let message_id = params.message_id();
    let is_admin = require_chat_access(&state, params.chat_id(), auth_user.id).await?;
    require_message_author(&state, params.chat_id(), message_id, auth_user.id, is_admin).await?;
    let view = view.into_inner();
    trigger_patch_message(state, message_id, view).await?;
    Ok(HttpResponse::Ok().finish())
}

impl From<AccessDenied> for PatchMessageError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => PatchMessageError::UnknownChat,
            AccessDenied::DatabaseError => PatchMessageError::DatabaseError,
        }
    }
}

impl From<MessageDenied> for PatchMessageError {
    fn from(denied: MessageDenied) -> Self {
        match denied {
            MessageDenied::NotFound => PatchMessageError::UnknownEvent,
            MessageDenied::Forbidden => PatchMessageError::Forbidden,
            MessageDenied::DatabaseError => PatchMessageError::DatabaseError,
        }
    }
}
