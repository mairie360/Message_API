use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::acknowledge_read::view::AcknowledgeReadQueryView;
use crate::endpoints::v1::id::access::{require_chat_access, AccessDenied};
use crate::endpoints::v1::id::read::post::view::{AcknowledgeReadResultView, AcknowledgeReadView};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum AcknowledgeReadError {
    DatabaseError,
    UnknownChat,
    UnknownMessage,
}

impl std::fmt::Display for AcknowledgeReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcknowledgeReadError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            AcknowledgeReadError::UnknownChat => write!(f, "Unknown chat."),
            AcknowledgeReadError::UnknownMessage => write!(f, "Unknown message."),
        }
    }
}

impl ResponseError for AcknowledgeReadError {
    fn status_code(&self) -> StatusCode {
        match self {
            AcknowledgeReadError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AcknowledgeReadError::UnknownChat | AcknowledgeReadError::UnknownMessage => {
                StatusCode::NOT_FOUND
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<AccessDenied> for AcknowledgeReadError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => AcknowledgeReadError::UnknownChat,
            AccessDenied::DatabaseError => AcknowledgeReadError::DatabaseError,
        }
    }
}

async fn trigger_acknowledge_read(
    state: web::Data<AppState>,
    chat_id: u64,
    user_id: u64,
    view: AcknowledgeReadView,
) -> Result<AcknowledgeReadResultView, AcknowledgeReadError> {
    let unread_count = state
        .get_smart_db()
        .fetch_scalar::<i32, _>(&AcknowledgeReadQueryView::new(
            chat_id,
            user_id,
            view.read_until_message_id(),
        ))
        .await
        .map_err(|e| match e {
            ApiLibError::Database(DbError::NotFound) => AcknowledgeReadError::UnknownMessage,
            e => {
                eprintln!("Acknowledge read error: {e}");
                AcknowledgeReadError::DatabaseError
            }
        })?;

    Ok(AcknowledgeReadResultView::new(unread_count))
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    summary = "Acknowledge the messages read",
    description = "Marks the messages of a chat as read by the caller, up to and including `readUntilMessageId`, and \
                   returns how many messages are still unread. This is the only way to lower the caller's \
                   `unread_count`: `GET /api/v1/{chat_id}/` and `GET /api/v1/` never do, however often they are polled.\n\n\
                   Send the id of the last message the agent has actually displayed, not the newest message of the \
                   chat: messages posted after it stay unread, so a message that arrives while the thread is open \
                   is never acknowledged by mistake.\n\n\
                   The route is idempotent and only moves forward: acknowledging an older message than a previous \
                   call, or the same one again, changes nothing and returns the current count. The author is taken \
                   from the JWT, and messages written by the caller themselves are never counted as unread.\n\n\
                   Only the members of the chat may call this route; administrators bypass the check (their count is \
                   always `0` when they are not members). A caller who is not a member gets the same `404` as for an \
                   unknown chat.",
    responses(
        (
            status = 200,
            description = "Acknowledgement recorded (or already recorded). The body holds the number of messages still unread by the caller.",
            body = AcknowledgeReadResultView,
            example = json!({ "unread_count": 2 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, `chat_id` not an integer, or `readUntilMessageId` missing or not between 1 and 9223372036854775807.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `readUntilMessageId`: must be a message id between 1 and 9223372036854775807")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`, or the caller is neither one of its members nor an administrator (both cases are deliberately indistinguishable); or `readUntilMessageId` is not a message of this chat (unknown id, or a message of another chat), in which case the body is `Unknown message.`",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown message.")
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
        content = AcknowledgeReadView,
        description = "Id of the last message the agent has displayed.",
        example = json!({ "readUntilMessageId": 118 })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[post("/")]
pub async fn acknowledge_read(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<AcknowledgeReadView>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, AcknowledgeReadError> {
    let chat_id = params.chat_id;
    require_chat_access(&state, chat_id, auth_user.id).await?;
    let result = trigger_acknowledge_read(state, chat_id, auth_user.id, view.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
