use actix_web::http::StatusCode;
use actix_web::{patch, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::endpoints::v1::id::messages::id::patch::view::PatchMessageView;
use crate::endpoints::v1::id::messages::id::MessagePathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum PatchMessageError {
    BadRequest,
    DatabaseError,
    UnknownEvent,
}

impl std::fmt::Display for PatchMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchMessageError::BadRequest => {
                write!(f, "Bad request.")
            }
            PatchMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PatchMessageError::UnknownEvent => {
                write!(f, "Unknown event.")
            }
        }
    }
}

impl ResponseError for PatchMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            PatchMessageError::BadRequest => StatusCode::BAD_REQUEST,
            PatchMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PatchMessageError::UnknownEvent => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_patch_message(
    state: web::Data<AppState>,
    chat_id: u64,
    message_id: u64,
    view: PatchMessageView,
) -> Result<(), PatchMessageError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(PatchMessageError::DatabaseError),
    };

    //query

    // update cache

    Ok(())
}

#[utoipa::path(
    patch,
    path = "",
    responses(
        (status = 204, description = "Message patched successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    params(
        MessagePathParams
    ),
    request_body = PatchMessageView,
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[patch("/")]
pub async fn patch_message(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
    view: web::Json<PatchMessageView>,
) -> Result<impl Responder, PatchMessageError> {
    let chat_id = params.chat_id();
    let message_id = params.message_id();
    let view = view.try_into().map_err(|_| PatchMessageError::BadRequest)?;
    trigger_patch_message(state, chat_id, message_id, view).await?;
    Ok(HttpResponse::NoContent().finish())
}
