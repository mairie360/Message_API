use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::unread_total::view::{
    UnreadTotalQueryResultView, UnreadTotalQueryView,
};
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::unread::get::view::UnreadTotalView;

#[derive(Debug, Clone, PartialEq)]
pub enum GetUnreadTotalError {
    DatabaseError,
}

impl std::fmt::Display for GetUnreadTotalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetUnreadTotalError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetUnreadTotalError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetUnreadTotalError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_unread_total(
    state: web::Data<AppState>,
    user_id: u64,
) -> Result<UnreadTotalView, GetUnreadTotalError> {
    let row: UnreadTotalQueryResultView = state
        .get_smart_db()
        .fetch_one(&UnreadTotalQueryView::new(user_id))
        .await
        .map_err(|e| {
            unexpected("read the unread total", e);
            GetUnreadTotalError::DatabaseError
        })?;
    // The counters never go below 0 (Database keeps them in step), so a negative value cannot
    // reach the client: clamp it rather than wrap.
    Ok(UnreadTotalView::new(
        u32::try_from(row.unread_count).unwrap_or(0),
        u32::try_from(row.unread_chats).unwrap_or(0),
    ))
}

#[utoipa::path(
    get,
    path = "",
    summary = "Count the caller's unread messages",
    description = "Total of the caller's unread messages over every chat of their list (`GET /api/v1/`), \
                   and the number of those chats with at least one. A hidden direct chat is not counted \
                   until a new message shows it again. One aggregate, whatever the number of chats: meant \
                   for a badge (navigation, dashboard) that does not need the list itself.\n\n\
                   Like `GET /api/v1/`, it never changes the counters: they are only lowered by \
                   `POST /api/v1/{chat_id}/read/`.",
    responses(
        (
            status = 200,
            description = "The caller's unread total (`0` / `0` when they have no chat or read everything).",
            body = UnreadTotalView,
            example = json!({ "unread_count": 7, "unread_chats": 3 })
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
pub async fn get_unread_total(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl Responder, GetUnreadTotalError> {
    let result = trigger_get_unread_total(state, auth_user.id).await?;
    Ok(HttpResponse::Ok().json(result))
}
