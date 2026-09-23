use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::patch_message_in_chat::view::PatchMessageQueryView;
use crate::endpoints::v1::id::messages::id::patch::view::PatchMessageView;
use crate::endpoints::v1::id::messages::id::MessagePathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchMessageError {
    DatabaseError,
    UnknownEvent,
}

impl std::fmt::Display for PatchMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    summary = "Modifier un message",
    description = "Remplace le contenu d'un message. `content` est obligatoire : ce `PATCH` \
                   n'est pas une modification partielle, il écrase le texte.\n\n\
                   Contrairement à la publication, aucun `ChatSignal` n'est poussé sur le flux \
                   SSE : les autres participants ne voient la correction qu'à leur prochain \
                   rechargement de la conversation.\n\n\
                   Aucun contrôle d'auteur ni d'appartenance : tout utilisateur authentifié peut \
                   modifier n'importe quel message. La réponse a un corps vide.",
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
    _: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
    view: ValidatedJson<PatchMessageView>,
) -> Result<impl Responder, PatchMessageError> {
    let message_id = params.message_id();
    let view = view.into_inner();
    trigger_patch_message(state, message_id, view).await?;
    Ok(HttpResponse::Ok().finish())
}
