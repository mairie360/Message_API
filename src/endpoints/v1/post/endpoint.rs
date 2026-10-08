use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::create_chat::view::CreateChatQueryView;
use crate::database::ids::id_from_sql;
use crate::endpoints::error::{classify, DbFailure};
use crate::endpoints::v1::post::view::{CreateChatResultView, CreateChatView};
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateChatError {
    DatabaseError,
    UnknownMember,
}

impl std::fmt::Display for CreateChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            CreateChatError::UnknownMember => {
                write!(f, "`members` contains an unknown user.")
            }
        }
    }
}

impl ResponseError for CreateChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            CreateChatError::UnknownMember => StatusCode::BAD_REQUEST,
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
    let mut members = view.members().to_vec();
    members.push(user_id);
    // The creator may also be listed: a duplicate would break the (conversation, user) key.
    members.sort_unstable();
    members.dedup();

    // One statement creates the chat and its members: an unknown member fails it as a whole.
    let query_view = CreateChatQueryView::with_members(view.name(), None, Some(user_id), &members);
    let chat_id = state
        .get_smart_db()
        .fetch_scalar::<i32, _>(&query_view)
        .await
        .map_err(|e| match classify("create chat", e) {
            DbFailure::ForeignKey(_) => CreateChatError::UnknownMember,
            _ => CreateChatError::DatabaseError,
        })?;

    Ok(CreateChatResultView::new(id_from_sql(chat_id)))
}

#[utoipa::path(
    post,
    path = "",
    summary = "Create a group chat",
    description = "Creates a **group** chat (`kind` `group` in `GET /api/v1/`) and attaches the caller and the users \
                   listed in `members`, in a single atomic operation: if one member does not exist, nothing is \
                   created.\n\n\
                   To talk to a single agent, open the direct chat of the pair with `POST /api/v1/direct/` instead: \
                   this route always creates a new chat, even with a single member.\n\n\
                   The caller is added automatically and becomes the **creator** of the chat: while a member, \
                   they are the only one (with administrators) who may add members or remove someone else. \
                   Listing the caller in `members` is harmless.\n\n\
                   Ids are Core API account ids. The response only holds the id given to the chat.",
    responses(
        (
            status = 200,
            description = "Chat created. The body holds its id.",
            body = CreateChatResultView,
            example = json!({ "id": 5 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, missing field, `name` not empty (no title) nor 1 to 150 characters without control characters, `members` longer than 50, with a duplicate or an id outside 1..=2147483647 (`Invalid `members`: …`), or `members` containing an unknown user (`` `members` contains an unknown user.``). No chat is created.",
            body = String,
            content_type = "text/plain",
            example = json!("`members` contains an unknown user.")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 500,
            description = "Database error. Nothing is created.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = CreateChatView,
        description = "Title of the chat and Core API ids of its other members.",
        example = json!({
            "name": "Service urbanisme",
            "members": [42, 51]
        })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[post("/")]
pub async fn create_chat(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<CreateChatView>,
) -> Result<impl Responder, CreateChatError> {
    let view = view.into_inner();
    let result = trigger_create_chat(state, auth_user.id, view).await?;
    Ok(HttpResponse::Ok().json(result))
}
