use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::add_users_to_chat::view::AddMembersToChatQueryView;
use crate::endpoints::v1::id::users::post::view::AddUsersToChat;
use crate::endpoints::v1::id::ChatPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum AddUsersToChatError {
    DatabaseError,
    BadRequest,
}

impl std::fmt::Display for AddUsersToChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddUsersToChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            AddUsersToChatError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for AddUsersToChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddUsersToChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AddUsersToChatError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_add_users_to_chat(
    state: web::Data<AppState>,
    chat_id: u64,
    view: AddUsersToChat,
) -> Result<(), AddUsersToChatError> {
    let view = AddMembersToChatQueryView::new(chat_id, view.users_id().to_vec());
    if view.is_empty() {
        return Ok(());
    }

    state
        .get_smart_db()
        .execute(view)
        .await
        .map_err(|_| AddUsersToChatError::DatabaseError)?;

    Ok(())
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    responses(
        (status = 200, description = "Users added to chat successfully"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    request_body = AddUsersToChat,
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[post("/")]
pub async fn add_users_to_chat(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    view: web::Json<AddUsersToChat>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, AddUsersToChatError> {
    let view = view
        .try_into()
        .map_err(|_| AddUsersToChatError::BadRequest)?;
    trigger_add_users_to_chat(state, params.chat_id, view).await?;
    Ok(HttpResponse::Ok().finish())
}
