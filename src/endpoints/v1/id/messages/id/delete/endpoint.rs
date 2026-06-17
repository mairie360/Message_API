use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::id::messages::id::MessagePathParams;
use crate::endpoints::v1::id::{messages, ChatPathParams};

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteMessageError {
    DatabaseError,
    UnknownEvent,
}

impl std::fmt::Display for DeleteMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteMessageError::UnknownEvent => {
                write!(f, "Unknown event.")
            }
        }
    }
}

impl ResponseError for DeleteMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteMessageError::UnknownEvent => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_message(
    state: web::Data<AppState>,
    chat_id: u64,
    message_id: u64,
) -> Result<(), DeleteMessageError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(DeleteMessageError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    responses(
        (status = 204, description = "Message deleted successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    params(
        MessagePathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[delete("/")]
pub async fn delete_message(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
) -> Result<impl Responder, DeleteMessageError> {
    let chat_id = params.chat_id();
    let message_id = params.message_id();
    trigger_delete_message(state, chat_id, message_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
