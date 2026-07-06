use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::chats::get_chats::query::get_chats_query;
use crate::database::chats::get_chats::view::GetChatsQueryView;
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
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(GetChatsError::DatabaseError),
    };

    // get cache

    let view = GetChatsQueryView::new(user_id);
    let result = get_chats_query(view, pool)
        .await
        .map_err(|_| GetChatsError::DatabaseError)?;

    // update cache

    Ok(result.into())
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "Chats retrieved successfully", body = GetChatsResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
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
