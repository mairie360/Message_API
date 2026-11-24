use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::database::chats::message_reactions::view::RemoveReactionQueryView;
use crate::endpoints::error::{classify, DbFailure};
use crate::endpoints::v1::id::access::{begin, commit, AccessDenied, NOT_A_MEMBER_MESSAGE};
use crate::endpoints::v1::id::messages::id::patch::endpoint::SYSTEM_NOTE_MESSAGE;
use crate::endpoints::v1::id::messages::id::reactions::delete::view::RemoveReactionQuery;
use crate::endpoints::v1::id::messages::id::reactions::post::view::ReactionsView;
use crate::endpoints::v1::id::messages::id::reactions::{
    prepare_reaction, reactions_of, ReactionDenied,
};
use crate::endpoints::v1::id::messages::id::MessagePathParams;
use crate::endpoints::validation::ValidatedQuery;
use crate::realtime::state::ChatEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum RemoveReactionError {
    DatabaseError,
    UnknownChat,
    NotAMember,
    UnknownMessage,
    SystemNote,
    /// The caller had not reacted with this emoji.
    UnknownReaction,
}

impl std::fmt::Display for RemoveReactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoveReactionError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            RemoveReactionError::UnknownChat => write!(f, "Unknown chat."),
            RemoveReactionError::NotAMember => f.write_str(NOT_A_MEMBER_MESSAGE),
            RemoveReactionError::UnknownMessage => write!(f, "Unknown message."),
            RemoveReactionError::SystemNote => f.write_str(SYSTEM_NOTE_MESSAGE),
            RemoveReactionError::UnknownReaction => write!(f, "Unknown reaction."),
        }
    }
}

impl ResponseError for RemoveReactionError {
    fn status_code(&self) -> StatusCode {
        match self {
            RemoveReactionError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            RemoveReactionError::UnknownChat
            | RemoveReactionError::UnknownMessage
            | RemoveReactionError::UnknownReaction => StatusCode::NOT_FOUND,
            RemoveReactionError::NotAMember => StatusCode::FORBIDDEN,
            RemoveReactionError::SystemNote => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_remove_reaction(
    tx: &mut SmartTransaction,
    params: &MessagePathParams,
    user_id: u64,
    query: &RemoveReactionQuery,
) -> Result<ReactionsView, RemoveReactionError> {
    prepare_reaction(tx, params.chat_id(), params.message_id(), user_id).await?;
    tx.fetch_scalar::<i64, _>(&RemoveReactionQueryView::new(
        params.message_id(),
        user_id,
        query.emoji(),
    ))
    .await
    .map_err(|e| match classify("remove reaction", e) {
        DbFailure::NotFound => RemoveReactionError::UnknownReaction,
        _ => RemoveReactionError::DatabaseError,
    })?;
    Ok(ReactionsView::new(
        reactions_of(tx, params.message_id()).await?,
    ))
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Remove a reaction",
    description = "Removes the caller's reaction `emoji` from a message (MAIR-1155) and answers the reactions left. \
                   `404 Unknown reaction.` when the caller had not reacted with it.\n\n \
                   Once committed, a `MSG_REACTED` `ChatSignal` (with `message_id`) is pushed to the other members.\n\n \
                   Same access rules as `POST`: members only (`403` for an administrator who is not a member, \
                   `404` for anyone else), `404` for a deleted message, `400` for a system note.",
    params(MessagePathParams, RemoveReactionQuery),
    responses(
        (
            status = 200,
            description = "Reaction removed: the reactions left on the message.",
            body = ReactionsView,
            example = json!({ "reactions": [{ "emoji": "👍", "count": 1, "user_ids": [51] }] })
        ),
        (
            status = 400,
            description = "A URL segment not an integer, `emoji` missing, empty, longer than 32 characters or holding whitespace or a control character, or the message is a system note (`A system note cannot be changed.`).",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `emoji`: must not be empty")
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
            description = "No chat matches `chat_id` or the caller is neither a member nor an administrator (`Unknown chat.`), no live message `message_id` in this chat (`Unknown message.`), or the caller had not reacted with `emoji` (`Unknown reaction.`).",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown reaction.")
        ),
        (
            status = 500,
            description = "Database error.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[delete("/")]
pub async fn remove_reaction(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::realtime::state::AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
    query: ValidatedQuery<RemoveReactionQuery>,
) -> Result<impl Responder, RemoveReactionError> {
    let mut tx = begin(&state).await?;
    let result =
        trigger_remove_reaction(&mut tx, &params, auth_user.id, &query.into_inner()).await?;
    commit(tx).await?;
    sse_state
        .publish(ChatEvent::message_reacted(
            params.chat_id(),
            auth_user.id,
            params.message_id(),
        ))
        .await;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for RemoveReactionError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => RemoveReactionError::UnknownChat,
            AccessDenied::DatabaseError => RemoveReactionError::DatabaseError,
        }
    }
}

impl From<ReactionDenied> for RemoveReactionError {
    fn from(denied: ReactionDenied) -> Self {
        match denied {
            ReactionDenied::UnknownChat => RemoveReactionError::UnknownChat,
            ReactionDenied::NotAMember => RemoveReactionError::NotAMember,
            ReactionDenied::UnknownMessage => RemoveReactionError::UnknownMessage,
            ReactionDenied::SystemNote => RemoveReactionError::SystemNote,
            ReactionDenied::DatabaseError => RemoveReactionError::DatabaseError,
        }
    }
}
