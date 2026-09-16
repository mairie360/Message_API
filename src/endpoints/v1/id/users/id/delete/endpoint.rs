use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::remove_user_from_chat::view::RemoveMemberFromChatQueryView;
use crate::endpoints::v1::id::users::id::UsersPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveUserFromChatError {
    DatabaseError,
    BadRequest,
}

impl std::fmt::Display for RemoveUserFromChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Retirer un participant d'une conversation",
    description = "Détache un utilisateur d'une conversation, qui cesse d'apparaître dans son \
                   `GET /api/v1/`. Ses messages déjà publiés sont conservés.\n\n\
                   Opération idempotente : retirer quelqu'un qui n'est pas participant répond \
                   également `204`.\n\n\
                   Aucun contrôle d'appartenance : tout utilisateur authentifié peut appeler cette route sur \
                   n'importe quelle conversation dont il connaît l'identifiant.",
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
    _: AuthenticatedUser,
    params: web::Path<UsersPathParams>,
) -> Result<impl Responder, RemoveUserFromChatError> {
    let chat_id = params.chat_id();
    let user_id = params.user_id();
    trigger_remove_user_from_chat(state, chat_id, user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
