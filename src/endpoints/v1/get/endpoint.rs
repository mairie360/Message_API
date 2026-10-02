use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::get_chats::view::{GetChatsQueryResultView, GetChatsQueryView};
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::get::view::GetChatsResultView;

#[derive(Debug, Clone, PartialEq)]
pub enum GetChatsError {
    DatabaseError,
}

impl std::fmt::Display for GetChatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetChatsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetChatsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetChatsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
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
    let result: Vec<GetChatsQueryResultView> =
        state.get_smart_db().fetch_all(&view).await.map_err(|e| {
            unexpected("get chats", e);
            GetChatsError::DatabaseError
        })?;

    Ok(result.into())
}

#[utoipa::path(
    get,
    path = "",
    summary = "List my chats",
    description = "Returns the chats the user of the JWT is a member of, with their number of unread \
                   messages in each.\n\n\
                   Reading `GET /api/v1/{chat_id}/` does not change `unread_count`: only the explicit \
                   `POST /api/v1/{chat_id}/read/` acknowledgement does.\n\n\
                   A chat without a title has an empty `name`, never `null`.",
    responses(
        (
            status = 200,
            description = "Chats of the caller, newest first.",
            body = GetChatsResultView,
            example = json!({
                "chats": [
                    { "id": 5, "name": "Service urbanisme", "unread_count": 3 },
                    { "id": 8, "name": "Astreinte week-end", "unread_count": 0 }
                ]
            })
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 500,
            description = "Database error (logged with its cause).",
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
