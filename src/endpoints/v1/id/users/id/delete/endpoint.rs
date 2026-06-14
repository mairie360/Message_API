use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::id::users::id::UsersPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveUserFromChatError {
    DatabaseError,
    UnknownEvent,
}

impl std::fmt::Display for RemoveUserFromChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoveUserFromChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            RemoveUserFromChatError::UnknownEvent => {
                write!(f, "Unknown event.")
            }
        }
    }
}

impl ResponseError for RemoveUserFromChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            RemoveUserFromChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            RemoveUserFromChatError::UnknownEvent => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_remove_user_from_chat(
    state: web::Data<AppState>,
    chat_id: u64,
    user_id: u64,
) -> Result<(), RemoveUserFromChatError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(RemoveUserFromChatError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    responses(
        (status = 204, description = "User removed successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    params(
        UsersPathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[delete("/")]
pub async fn remove_user_from_chat(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<UsersPathParams>,
) -> Result<impl Responder, RemoveUserFromChatError> {
    let chat_id = params.chat_id();
    let user_id = params.user_id();
    trigger_remove_user_from_chat(state, chat_id, user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
