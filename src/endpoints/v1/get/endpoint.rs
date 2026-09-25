use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::get_chats::view::{GetChatsQueryResultView, GetChatsQueryView};
use crate::endpoints::v1::get::view::GetChatsResultView;

#[derive(Debug, Clone, PartialEq)]
pub enum GetChatsError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetChatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetChatsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetChatsError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetChatsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetChatsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetChatsError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_chats(
    state: web::Data<AppState>,
    user_id: u64,
) -> Result<GetChatsResultView, GetChatsError> {
    let view = GetChatsQueryView::new(user_id);
    let result: Vec<GetChatsQueryResultView> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetChatsError::DatabaseError)?;

    Ok(result.into())
}

#[utoipa::path(
    get,
    path = "",
    summary = "Lister ses conversations",
    description = "Renvoie les conversations dont l'utilisateur porté par le JWT est \
                   participant, avec son nombre de messages non lus sur chacune. C'est la seule \
                   route de cette API filtrée sur l'appelant.\n\n\
                   La lecture de `GET /api/v1/{chat_id}/` ne modifie pas \
                   `unread_count` : seul un acquittement explicite pourra le faire.\n\n\
                   Une conversation sans titre renvoie une chaîne vide, jamais `null`.",
    responses(
        (
            status = 200,
            description = "Conversations de l'utilisateur connecté.",
            body = GetChatsResultView,
            example = json!({
                "chats": [
                    { "id": 5, "name": "Service urbanisme", "unread_count": 3 },
                    { "id": 8, "name": "Astreinte week-end", "unread_count": 0 }
                ]
            })
        ),
        (
            status = 400,
            description = "Échec de la lecture en base. Ce endpoint renvoie `400` là où les autres renverraient `500`.",
            body = String,
            content_type = "text/plain",
            example = json!("Bad request.")
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
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[get("/")]
pub async fn get_chats(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetChatsError> {
    let result = trigger_get_chats(state, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(result))
}
