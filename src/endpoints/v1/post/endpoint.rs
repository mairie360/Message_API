use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::pool::AppState;
use mairie360_api_lib::security::AuthenticatedUser;

use crate::database::chats::add_users_to_chat::query::add_members_to_chat_query;
use crate::database::chats::add_users_to_chat::view::AddMembersToChatQueryView;
use crate::database::chats::create_chat::query::create_chat_query;
use crate::database::chats::create_chat::view::CreateChatQueryView;
use crate::endpoints::v1::post::view::{CreateChatResultView, CreateChatView};

#[derive(Debug, Clone, PartialEq)]
pub enum CreateChatError {
    DatabaseError,
    BadRequest,
}

impl std::fmt::Display for CreateChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            CreateChatError::BadRequest => {
                write!(f, "Bad request.")
            }
        }
    }
}

impl ResponseError for CreateChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            CreateChatError::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_create_chat(
    state: web::Data<AppState>,
    user_id: u64,
    view: CreateChatView,
) -> Result<CreateChatResultView, CreateChatError> {
    let pool = match state.db_pool.clone() {
        Some(pool) => pool,
        None => return Err(CreateChatError::DatabaseError),
    };

    let query_view = CreateChatQueryView::new(view.name(), None);
    let result = create_chat_query(query_view, pool.clone())
        .await
        .map_err(|_| CreateChatError::DatabaseError)? as u64;

    let mut members = view.members().to_vec();
    members.push(user_id);
    let query_view = AddMembersToChatQueryView::new(result, members);
    let _ = add_members_to_chat_query(query_view, pool.clone())
        .await
        .map_err(|_| CreateChatError::DatabaseError)?;

    // update cache

    Ok(CreateChatResultView::new(result))
}

#[utoipa::path(
    post,
    path = "",
    responses(
        (status = 200, description = "Chat created successfully", body = CreateChatResultView),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    request_body = CreateChatView,
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[post("/")]
pub async fn create_chat(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: web::Json<CreateChatView>,
) -> Result<impl Responder, CreateChatError> {
    let view = view.try_into().map_err(|_| CreateChatError::BadRequest)?;
    let result = trigger_create_chat(state, auth_user.id, view).await?;
    Ok(HttpResponse::Ok().json(result))
}
