use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::chats::delete_chat::query::delete_chat_query;
use crate::database::chats::delete_chat::view::DeleteChatQueryView;
use crate::endpoints::v1::id::ChatPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteChatError {
    DatabaseError,
    NothingToDelete,
    UnknownEvent,
}

impl std::fmt::Display for DeleteChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteChatError::NothingToDelete => {
                write!(f, "Nothing to delete.")
            }
            DeleteChatError::UnknownEvent => {
                write!(f, "Unknown event.")
            }
        }
    }
}

impl ResponseError for DeleteChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteChatError::NothingToDelete => StatusCode::OK,
            DeleteChatError::UnknownEvent => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_chat(
    state: web::Data<AppState>,
    chat_id: u64,
) -> Result<(), DeleteChatError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(DeleteChatError::DatabaseError),
    };

    let view = DeleteChatQueryView::new(chat_id);
    let result = delete_chat_query(view, pool.clone())
        .await
        .map_err(|_| DeleteChatError::DatabaseError)?;

    if result != 1 {
        return Err(DeleteChatError::DatabaseError);
    }

    // update cache

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    responses(
        (status = 204, description = "chat deleted successfully"),
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
#[delete("/")]
pub async fn delete_chat(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, DeleteChatError> {
    let chat_id = params.chat_id;
    trigger_delete_chat(state, chat_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
