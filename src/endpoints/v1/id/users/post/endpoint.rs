use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{
    begin, commit, require_chat_access_in, AccessDenied, NOT_A_MANAGER_MESSAGE,
};

use crate::database::chats::add_users_to_chat::view::AddMembersToChatQueryView;
use crate::endpoints::v1::id::users::post::view::{AddUsersToChat, AddUsersToChatResultView};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum AddUsersToChatError {
    DatabaseError,
    UnknownChat,
    UnknownUser,
    AlreadyMember,
    Forbidden,
}

impl std::fmt::Display for AddUsersToChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddUsersToChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            AddUsersToChatError::UnknownChat => write!(f, "Unknown chat."),
            AddUsersToChatError::Forbidden => f.write_str(NOT_A_MANAGER_MESSAGE),
            AddUsersToChatError::UnknownUser => write!(f, "`users_id` contains an unknown user."),
            AddUsersToChatError::AlreadyMember => {
                write!(f, "A user of `users_id` is already a member of this chat.")
            }
        }
    }
}

impl ResponseError for AddUsersToChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddUsersToChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AddUsersToChatError::UnknownChat => StatusCode::NOT_FOUND,
            AddUsersToChatError::UnknownUser => StatusCode::BAD_REQUEST,
            AddUsersToChatError::AlreadyMember => StatusCode::CONFLICT,
            AddUsersToChatError::Forbidden => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_add_users_to_chat(
    tx: &mut SmartTransaction,
    chat_id: u64,
    view: AddUsersToChat,
) -> Result<AddUsersToChatResultView, AddUsersToChatError> {
    let added = view.users_id().to_vec();
    let view = AddMembersToChatQueryView::new(chat_id, added.clone());
    if view.is_empty() {
        return Ok(AddUsersToChatResultView::new(chat_id, Vec::new()));
    }

    tx.execute(&view).await.map_err(|e| match e {
        // Both foreign keys of conversation_members: tell the missing chat from the missing user.
        ApiLibError::Database(DbError::ForeignKeyViolation(message))
            if message.contains("conversation_id") =>
        {
            AddUsersToChatError::UnknownChat
        }
        ApiLibError::Database(DbError::ForeignKeyViolation(_)) => AddUsersToChatError::UnknownUser,
        ApiLibError::Database(DbError::UniqueViolation(_)) => AddUsersToChatError::AlreadyMember,
        e => {
            eprintln!("Add chat members error: {e}");
            AddUsersToChatError::DatabaseError
        }
    })?;

    Ok(AddUsersToChatResultView::new(chat_id, added))
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    summary = "Add members to a chat",
    description = "Adds one or more users to a chat in a single call; the chat, **with its whole history**, \
                   then shows up in their `GET /api/v1/`. The call is all or nothing: one unknown user or one \
                   user already member and nobody is added.\n\n \
                   **Only the creator of the chat (while still a member) and administrators** may add users; \
                   any other member gets `403`. A caller who is neither a member nor an administrator gets \
                   the same `404` as for an unknown chat.",
    responses(
        (
            status = 200,
            description = "Users added. `added` lists them (every id of `users_id`).",
            body = AddUsersToChatResultView,
            example = json!({ "chat_id": 5, "added": [42, 51] })
        ),
        (
            status = 400,
            description = "Malformed JSON body, `chat_id` not an integer, `users_id` missing, empty, longer than 50, with a duplicate or an id outside 1..=2147483647 (`Invalid `users_id`: …`), or containing an unknown user.",
            body = String,
            content_type = "text/plain",
            example = json!("`users_id` contains an unknown user.")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "The caller is a member but neither the creator of the chat nor an administrator.",
            body = String,
            content_type = "text/plain",
            example = json!("Only the creator of the chat or an administrator can manage its other members.")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`, or the caller is neither one of its members nor an administrator (both cases are deliberately indistinguishable).",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown chat.")
        ),
        (
            status = 409,
            description = "A user of `users_id` is already a member of the chat; nobody is added.",
            body = String,
            content_type = "text/plain",
            example = json!("A user of `users_id` is already a member of this chat.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = AddUsersToChat,
        description = "Core API ids of the users to attach.",
        example = json!({ "users_id": [42, 51] })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[post("/")]
pub async fn add_users_to_chat(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<AddUsersToChat>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, AddUsersToChatError> {
    let view = view.into_inner();
    let mut tx = begin(&state).await?;
    let role = require_chat_access_in(&mut tx, params.chat_id, auth_user.id).await?;
    if !role.can_manage_members() {
        return Err(AddUsersToChatError::Forbidden);
    }
    let result = trigger_add_users_to_chat(&mut tx, params.chat_id, view).await?;
    commit(tx).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for AddUsersToChatError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => AddUsersToChatError::UnknownChat,
            AccessDenied::DatabaseError => AddUsersToChatError::DatabaseError,
        }
    }
}
