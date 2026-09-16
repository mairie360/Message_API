use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::delete_message_from_chat::view::DeleteMessageQueryView;
use crate::endpoints::v1::id::messages::id::MessagePathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteMessageError {
    DatabaseError,
    UnknownMessage,
}

impl std::fmt::Display for DeleteMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    summary = "Supprimer un message",
    description = "Supprime définitivement un message d'une conversation.\n\n\
                   Aucun `ChatSignal` n'est poussé sur le flux SSE : les autres participants \
                   voient encore le message jusqu'à leur prochain rechargement.\n\n\
                   Aucun contrôle d'auteur ni d'appartenance : tout utilisateur authentifié peut \
                   supprimer n'importe quel message.",
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
            status = 404,
            description = "Aucun message ne porte cet identifiant.",
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
    _: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
) -> Result<impl Responder, DeleteMessageError> {
    let message_id = params.message_id();
    trigger_delete_message(state, message_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
