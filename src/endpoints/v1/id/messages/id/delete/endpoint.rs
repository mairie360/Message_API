use actix_web::http::StatusCode;
use actix_web::{delete, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{
    begin, commit, require_chat_access_in, require_message_author_in, AccessDenied, MessageDenied,
};

use crate::database::chats::delete_message_from_chat::view::{
    DeleteMessageQueryView, LockChatMessagesQueryView,
};
use crate::database::chats::message_reactions::view::{MessageState, MessageStateQueryView};
use crate::database::chats::message_tags::view::PurgeMessageExtrasQueryView;
use crate::endpoints::error::{classify, unexpected, DbFailure};
use crate::endpoints::v1::id::messages::id::patch::endpoint::SYSTEM_NOTE_MESSAGE;
use crate::endpoints::v1::id::messages::id::MessagePathParams;
use crate::realtime::state::ChatEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteMessageError {
    DatabaseError,
    UnknownMessage,
    Forbidden,
    /// The message is a system note (MAIR-1155).
    SystemNote,
}

impl std::fmt::Display for DeleteMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteMessageError::Forbidden => {
                write!(f, "Only the author of a message can delete it.")
            }
            DeleteMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            DeleteMessageError::UnknownMessage => {
                write!(f, "Unknown message.")
            }
            DeleteMessageError::SystemNote => f.write_str(SYSTEM_NOTE_MESSAGE),
        }
    }
}

impl ResponseError for DeleteMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteMessageError::Forbidden => StatusCode::FORBIDDEN,
            DeleteMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            DeleteMessageError::UnknownMessage => StatusCode::NOT_FOUND,
            DeleteMessageError::SystemNote => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_delete_message(
    tx: &mut SmartTransaction,
    chat_id: u64,
    message_id: u64,
    performed_by: u64,
) -> Result<(), DeleteMessageError> {
    let state: MessageState = tx
        .fetch_one(&MessageStateQueryView::new(chat_id, message_id))
        .await
        .map_err(|e| match classify("message state", e) {
            DbFailure::NotFound => DeleteMessageError::UnknownMessage,
            _ => DeleteMessageError::DatabaseError,
        })?;
    if state.is_system() {
        return Err(DeleteMessageError::SystemNote);
    }
    // The unread counters it lowers must not race an acknowledgement of the chat.
    tx.execute(&LockChatMessagesQueryView::new(chat_id))
        .await
        .map_err(|e| {
            unexpected("lock chat messages", e);
            DeleteMessageError::DatabaseError
        })?;
    // Logs the deletion in messaging_moderation_log when the message is someone else's.
    let view = DeleteMessageQueryView::new(chat_id, message_id, performed_by);
    tx.fetch_scalar::<i64, _>(&view)
        .await
        .map_err(|e| match classify("delete message", e) {
            DbFailure::NotFound => DeleteMessageError::UnknownMessage,
            _ => DeleteMessageError::DatabaseError,
        })?;
    // A tombstone keeps no reaction, mention nor link (MAIR-1155). A statement of its own: it
    // also sees a reaction committed while the deletion waited for the row.
    tx.execute(&PurgeMessageExtrasQueryView::new(message_id))
        .await
        .map_err(|e| {
            unexpected("purge message extras", e);
            DeleteMessageError::DatabaseError
        })?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "",
    summary = "Delete a message",
    description = "Deletes a message of a chat. The message becomes a **tombstone** (MAIR-524): its content is erased, \
                   the history (`GET /api/v1/{chat_id}/`) no longer returns it nor its excerpt in the messages that \
                   quote it, it cannot be edited, quoted nor deleted again (`404`), and the unread counters that \
                   still counted it are lowered. The incremental read `GET /api/v1/{chat_id}/?since=` returns it \
                   once as `deleted: true` so that the clients remove it.\n\n \
                   Once committed, a `MSG_DELETED` `ChatSignal` (with `message_id`) is pushed on the WebSocket of \
                   every other connected member.\n\n \
                   Only the author of the message may delete it (`403` for another member). Administrators may \
                   delete any message, even in a chat they are not a member of (moderation): the deletion of a \
                   message written by someone else is recorded, with its content, in the moderation log.\n\n \
                   The reactions, mentions and links of the message are deleted with it (MAIR-1155). A system \
                   note (the record of a member change) cannot be deleted (`400`).",
    responses(
        (
            status = 204,
            description = "Message deleted. Empty body.",
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, or the message is a system note (`A system note cannot be changed.`, MAIR-1155).",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid path: ids are integers.")
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
            description = "The message was written by another member and the caller is not an administrator.",
            body = String,
            content_type = "text/plain",
            example = json!("Only the author of a message can delete it.")
        ),
        (
            status = 404,
            description = "No message `message_id` in this chat, the message is already deleted, or the caller is neither a member of the chat nor an administrator.",
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
    params(
        MessagePathParams
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[delete("/")]
pub async fn delete_message(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::realtime::state::AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
) -> Result<impl Responder, DeleteMessageError> {
    let message_id = params.message_id();
    let mut tx = begin(&state).await?;
    let role = require_chat_access_in(&mut tx, params.chat_id(), auth_user.id).await?;
    require_message_author_in(
        &mut tx,
        params.chat_id(),
        message_id,
        auth_user.id,
        role.is_admin,
    )
    .await?;
    trigger_delete_message(&mut tx, params.chat_id(), message_id, auth_user.id).await?;
    commit(tx).await?;
    // Only once committed: the members drop the message as soon as they get the signal.
    sse_state
        .publish(ChatEvent::message_deleted(
            params.chat_id(),
            auth_user.id,
            message_id,
        ))
        .await;
    Ok(HttpResponse::NoContent().finish())
}

impl From<AccessDenied> for DeleteMessageError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => DeleteMessageError::UnknownMessage,
            AccessDenied::DatabaseError => DeleteMessageError::DatabaseError,
        }
    }
}

impl From<MessageDenied> for DeleteMessageError {
    fn from(denied: MessageDenied) -> Self {
        match denied {
            MessageDenied::NotFound => DeleteMessageError::UnknownMessage,
            MessageDenied::Forbidden => DeleteMessageError::Forbidden,
            MessageDenied::DatabaseError => DeleteMessageError::DatabaseError,
        }
    }
}
