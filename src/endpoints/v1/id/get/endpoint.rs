use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{require_chat_access, AccessDenied};

use crate::database::chats::get_chat::view::{GetChatQueryView, Message};
use crate::endpoints::v1::id::get::view::GetChatResultView;
use crate::endpoints::v1::id::ChatPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetChatError {
    DatabaseError,
    UnknownChat,
    NotFound,
}

impl std::fmt::Display for GetChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetChatError::NotFound => write!(f, "Unknown chat."),
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
            GetChatError::NotFound => StatusCode::NOT_FOUND,
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
    summary = "Read the messages of a chat",
    description = "Returns the messages of a chat **without modifying** the caller's unread counter. An explicit \
                   acknowledgement will be provided by a separate write route.\n\n\
                   There is no pagination: every message is returned.\n\n\
                   Only the members of the chat may call this route; administrators bypass the check. A caller who is not a member gets the same `404` as for an unknown chat.",
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
    user: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, GetChatError> {
    let chat_id = params.chat_id;
    require_chat_access(&state, chat_id, user.id).await?;
    let result = trigger_get_chat(state, chat_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetChatError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetChatError::NotFound,
            AccessDenied::DatabaseError => GetChatError::DatabaseError,
        }
    }
}
