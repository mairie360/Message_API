use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::chats::get_chat_users::query::get_chat_members_query;
use crate::database::chats::get_chat_users::view::GetChatMembersQueryView;
use crate::endpoints::v1::id::users::get::view::{GetUsersView, User};
use crate::endpoints::v1::id::ChatPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetChatUsersError {
    BadRequest,
    DatabaseError,
}

impl std::fmt::Display for GetChatUsersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetChatUsersError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetChatUsersError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for GetChatUsersError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetChatUsersError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetChatUsersError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_chat_users(
    state: web::Data<AppState>,
    chat_id: u64,
) -> Result<GetUsersView, GetChatUsersError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetChatUsersError::DatabaseError),
    };

    // get cache

    let view = GetChatMembersQueryView::new(chat_id);
    let result = get_chat_members_query(view, pool)
        .await
        .map_err(|_| GetChatUsersError::DatabaseError)?;

    // update cache

    Ok(GetUsersView::new(
        result
            .into_iter()
            .map(|user_id| User::new(user_id as u64))
            .collect(),
    ))
}

#[utoipa::path(
    get,
    params(
        ChatPathParams,
    ),
    path = "",
    responses(
        (status = 200, description = "Chat users retrieved successfully", body = GetUsersView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[get("/")]
pub async fn get_chat_users(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, GetChatUsersError> {
    let chat_id = params.chat_id;
    let result = trigger_get_chat_users(state, chat_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
