use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::chats::add_message_to_chat::query::post_message_in_chat_query;
use crate::database::chats::add_message_to_chat::view::PostMessageInChatQueryView;
use crate::endpoints::v1::id::messages::post::view::{PostMessageResultView, PostMessageView};
use crate::endpoints::v1::id::ChatPathParams;
use crate::sse::state::ChatEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum PosteMessageError {
    DatabaseError,
    BadRequest,
}

impl std::fmt::Display for PosteMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PosteMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PosteMessageError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for PosteMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            PosteMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PosteMessageError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_post_message(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::sse::state::AppState>,
    user_id: u64,
    view: PostMessageView,
    chat_id: u64,
) -> Result<PostMessageResultView, PosteMessageError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(PosteMessageError::DatabaseError),
    };

    let view = PostMessageInChatQueryView::new(chat_id, user_id, view.content());
    let chat_event = ChatEvent {
        chat_id,
        sender_id: user_id,
        message: view.message().to_string(),
    };
    let result = post_message_in_chat_query(view, pool)
        .await
        .map_err(|_| PosteMessageError::DatabaseError)?;

    let _ = sse_state.internal_bus.send(chat_event);

    // update cache

    Ok(PostMessageResultView::new(result as u64))
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    responses(
        (status = 200, description = "Message posted successfully", body = PostMessageResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    request_body = PostMessageView,
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[post("/")]
pub async fn post_message(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::sse::state::AppState>,
    auth_user: AuthenticatedUser,
    view: web::Json<PostMessageView>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, PosteMessageError> {
    let view = view.try_into().map_err(|_| PosteMessageError::BadRequest)?;
    let chat_id = params.chat_id;
    let result = trigger_post_message(state, sse_state, auth_user.id, view, chat_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
