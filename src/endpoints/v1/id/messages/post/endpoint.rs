use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{
    begin, commit, require_chat_access_in, AccessDenied, NotAMember, NOT_A_MEMBER_MESSAGE,
};

use crate::database::chats::add_message_to_chat::view::PostMessageInChatQueryView;
use crate::endpoints::v1::id::messages::post::view::{PostMessageResultView, PostMessageView};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedJson;
use crate::sse::state::ChatEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum PosteMessageError {
    DatabaseError,
    UnknownChat,
    UnknownCitation,
    NotAMember,
}

impl std::fmt::Display for PosteMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PosteMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PosteMessageError::UnknownChat => write!(f, "Unknown chat."),
            PosteMessageError::UnknownCitation => {
                write!(f, "`citation` is not a message of this chat.")
            }
            PosteMessageError::NotAMember => f.write_str(NOT_A_MEMBER_MESSAGE),
        }
    }
}

impl ResponseError for PosteMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            PosteMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PosteMessageError::UnknownChat => StatusCode::NOT_FOUND,
            PosteMessageError::UnknownCitation => StatusCode::BAD_REQUEST,
            PosteMessageError::NotAMember => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_post_message(
    tx: &mut SmartTransaction,
    user_id: u64,
    view: PostMessageView,
    chat_id: u64,
) -> Result<PostMessageResultView, PosteMessageError> {
    let view =
        PostMessageInChatQueryView::replying_to(chat_id, user_id, view.content(), view.citation());
    let result = tx
        .fetch_scalar::<i64, _>(&view)
        .await
        .map_err(|e| match e {
            // fk_messages_reply_to: the quoted message is unknown or in another chat.
            ApiLibError::Database(DbError::ForeignKeyViolation(message))
                if message.contains("reply_to") =>
            {
                PosteMessageError::UnknownCitation
            }
            ApiLibError::Database(DbError::ForeignKeyViolation(_)) => {
                PosteMessageError::UnknownChat
            }
            e => {
                eprintln!("Post message error: {e}");
                PosteMessageError::DatabaseError
            }
        })?;

    Ok(PostMessageResultView::new(result as u64))
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    summary = "Post a message",
    description = "Adds a message to a chat and pushes a `ChatSignal` on the SSE stream of every other connected \
                   member. The author is taken from the JWT, never from the body.\n\n \
                   `citation` is optional: the id of the message this one answers. It must be a message of the \
                   same chat (`400` otherwise) and is read back as `citation` by `GET /api/v1/{chat_id}/` \
                   (`null` once the quoted message is deleted).\n\n \
                   The response only holds the id given to the message.\n\n \
                   Only the members of the chat may post. An administrator who is not a member gets `403` \
                   (administrators read and moderate, they do not take part); anyone else gets the same `404` \
                   as for an unknown chat.",
    responses(
        (
            status = 200,
            description = "Message posted. The body holds its id.",
            body = PostMessageResultView,
            example = json!({ "id": 101 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, `chat_id` not an integer, `content` breaking its rules (not blank, at most 5000 characters, no `<` / `>`, no control character other than line breaks and tabs), `citation` above 9223372036854775807, or `citation` not a message of this chat (`` `citation` is not a message of this chat.``).",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `content`: must not contain `<` or `>`")
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
            description = "The caller is an administrator who is not a member of the chat.",
            body = String,
            content_type = "text/plain",
            example = json!("Administrators may only read and moderate a chat they are not a member of.")
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
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = PostMessageView,
        description = "Content of the message and, optionally, the message it answers.",
        example = json!({ "content": "La réunion est décalée à 15h.", "citation": 118 })
    ),
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
    view: ValidatedJson<PostMessageView>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, PosteMessageError> {
    let view = view.into_inner();
    let chat_id = params.chat_id;
    let mut tx = begin(&state).await?;
    require_chat_access_in(&mut tx, chat_id, auth_user.id)
        .await?
        .require_member()?;
    let result = trigger_post_message(&mut tx, auth_user.id, view, chat_id).await?;
    commit(tx).await?;
    // Only once committed: the members reload the chat as soon as they get the signal.
    sse_state
        .publish(ChatEvent {
            chat_id,
            sender_id: auth_user.id,
        })
        .await;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for PosteMessageError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => PosteMessageError::UnknownChat,
            AccessDenied::DatabaseError => PosteMessageError::DatabaseError,
        }
    }
}

impl From<NotAMember> for PosteMessageError {
    fn from(_: NotAMember) -> Self {
        PosteMessageError::NotAMember
    }
}
