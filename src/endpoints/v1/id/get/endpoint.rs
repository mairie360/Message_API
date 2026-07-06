use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::chats::get_chat::query::get_chat_query;
use crate::database::chats::get_chat::view::GetChatQueryView;
use crate::database::chats::reset_unread_count::query::reset_unread_count_query;
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
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetChatError::DatabaseError),
    };

    //get chat from cache

    let view = GetChatQueryView::new(chat_id);
    let result = get_chat_query(view, pool.clone())
        .await
        .map_err(|_| GetChatError::DatabaseError)?;

    let view = ResetUnreadCountQueryView::new(chat_id, user_id);
    let _ = reset_unread_count_query(view, pool)
        .await
        .map_err(|_| GetChatError::DatabaseError)?;

    // update cache

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
