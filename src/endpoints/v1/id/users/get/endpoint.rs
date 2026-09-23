use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{require_chat_access, AccessDenied};

use crate::database::chats::get_chat_users::view::GetChatMembersQueryView;
use crate::endpoints::v1::id::users::get::view::{GetUsersView, User};
use crate::endpoints::v1::id::ChatPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum GetChatUsersError {
    BadRequest,
    DatabaseError,
    NotFound,
}

impl std::fmt::Display for GetChatUsersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetChatUsersError::NotFound => write!(f, "Unknown chat."),
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
            GetChatUsersError::NotFound => StatusCode::NOT_FOUND,
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
    let view = GetChatMembersQueryView::new(chat_id);
    let result: Vec<i32> = state
        .get_smart_db()
        .fetch_all(&view)
        .await
        .map_err(|_| GetChatUsersError::DatabaseError)?;

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
    summary = "List the members of a chat",
    description = "Returns the Core API ids of the members. Only ids are returned: pass them to \
                   `GET /api/v1/user/?ids=1,2,3` of Core API to get their names.\n\nOnly the members of the chat may call this route; administrators bypass the check. A caller who is not a member gets the same `404` as for an unknown chat.",
    responses(
        (
            status = 200,
            description = "Participants de la conversation.",
            body = GetUsersView,
            example = json!({ "users": [{ "id": 42 }, { "id": 51 }] })
        ),
        (
            status = 400,
            description = "Un segment de l'URL n'est pas un entier, ou le corps JSON est malformé.",
            body = String,
            content_type = "text/plain",
            example = json!("Path deserialize error: can not parse `abc` to a u64")
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`, or the caller is neither one of its members nor an administrator (both cases are deliberately indistinguishable).",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown chat.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[get("/")]
pub async fn get_chat_users(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, GetChatUsersError> {
    let chat_id = params.chat_id;
    require_chat_access(&state, chat_id, auth_user.id).await?;
    let result = trigger_get_chat_users(state, chat_id).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetChatUsersError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetChatUsersError::NotFound,
            AccessDenied::DatabaseError => GetChatUsersError::DatabaseError,
        }
    }
}
