use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::get_chat::view::{GetChatQueryView, Message};
use crate::database::chats::reset_unread_count::view::ResetUnreadCountQueryView;
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
    user_id: u64,
) -> Result<GetChatResultView, GetChatError> {
    let db = state.get_smart_db();

    let view = GetChatQueryView::new(chat_id);
    let result: Vec<Message> = db
        .fetch_all(&view)
        .await
        .map_err(|_| GetChatError::DatabaseError)?;

    let view = ResetUnreadCountQueryView::new(chat_id, user_id);
    db.execute(view)
        .await
        .map_err(|_| GetChatError::DatabaseError)?;

    Ok(result.into())
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Chat retrieved successfully", body = GetChatResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
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
    let result = trigger_get_chat(state, chat_id, user.id).await?;
    Ok(HttpResponse::Ok().json(result))
}
