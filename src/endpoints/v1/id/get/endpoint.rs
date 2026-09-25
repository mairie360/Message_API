use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::get_chat::view::{GetChatQueryView, Message};
use crate::endpoints::v1::id::get::view::GetChatResultView;
use crate::endpoints::v1::id::ChatPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetChatError {
    DatabaseError,
    UnknownChat,
}

impl std::fmt::Display for GetChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetChatError::UnknownChat => {
                write!(f, "Unknown chat.")
            }
        }
    }
}

impl ResponseError for GetChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetChatError::UnknownChat => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_chat(
    state: web::Data<AppState>,
    chat_id: u64,
) -> Result<GetChatResultView, GetChatError> {
    let db = state.get_smart_db();

    let view = GetChatQueryView::new(chat_id);
    let result: Vec<Message> = db
        .fetch_all(&view)
        .await
        .map_err(|_| GetChatError::DatabaseError)?;

    Ok(result.into())
}

#[utoipa::path(
    get,
    path = "",
    summary = "Lire les messages d'une conversation",
    description = "Renvoie les messages d'une conversation **sans modifier** le compteur de \
                   non-lus de l'appelant. Un acquittement explicite sera fourni par une route \
                   d'écriture distincte.\n\n\
                   Il n'y a pas de pagination : tous les messages sont renvoyés.\n\n\
                   Un `chat_id` inconnu renvoie une liste vide, pas une erreur.\n\n\
                   Aucun contrôle d'appartenance : tout utilisateur authentifié peut appeler cette route sur \
                   n'importe quelle conversation dont il connaît l'identifiant.",
    responses(
        (
            status = 200,
            description = "Messages de la conversation.",
            body = GetChatResultView,
            example = json!({
                "messages": [
                    {
                        "id": 101,
                        "content": "La réunion est décalée à 15h.",
                        "sender_id": 42,
                        "created_at": "2026-09-16T09:12:00Z",
                        "sitation": null
                    }
                ]
            })
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
        ChatPathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[get("/")]
pub async fn get_chat(
    state: web::Data<AppState>,
    _user: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, GetChatError> {
    let chat_id = params.chat_id;
    let result = trigger_get_chat(state, chat_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
