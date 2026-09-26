//! Chat membership check shared by every `/api/v1/{chat_id}/**` route.
//!
//! Only the (not excluded) members of a chat may read it, post in it and manage its members, and
//! only its author may edit or delete a message; administrators bypass both checks. A caller who may not see the chat gets the same answer as for
//! an unknown chat, so the routes never reveal that a chat exists.

use actix_web::web;
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::state::AppState;

use crate::database::chats::access::view::{ChatAccess, ChatAccessQueryView};
use crate::database::chats::message_owner::view::MessageOwnerQueryView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDenied {
    /// The chat does not exist, or the caller is neither a member nor an administrator.
    NotFound,
    DatabaseError,
}

/// Checks that `user_id` may act on chat `chat_id` and returns whether they are an administrator
/// (who may also act on the messages of others and delete the chat).
pub async fn require_chat_access(
    state: &web::Data<AppState>,
    chat_id: u64,
    user_id: u64,
) -> Result<bool, AccessDenied> {
    let access: ChatAccess = state
        .get_smart_db()
        .fetch_one(&ChatAccessQueryView::new(chat_id, user_id))
        .await
        .map_err(|e| {
            eprintln!("Chat access error: {e}");
            AccessDenied::DatabaseError
        })?;
    if !access.chat_exists || !(access.is_member || access.is_admin) {
        return Err(AccessDenied::NotFound);
    }
    Ok(access.is_admin)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDenied {
    /// No message `message_id` in this chat.
    NotFound,
    /// The message belongs to another member.
    Forbidden,
    DatabaseError,
}

/// Checks that message `message_id` belongs to chat `chat_id` and that `user_id` wrote it; an
/// administrator (`is_admin`, from [`require_chat_access`]) may act on any message.
pub async fn require_message_author(
    state: &web::Data<AppState>,
    chat_id: u64,
    message_id: u64,
    user_id: u64,
    is_admin: bool,
) -> Result<(), MessageDenied> {
    let owner_id: i32 = state
        .get_smart_db()
        .fetch_scalar(&MessageOwnerQueryView::new(chat_id, message_id))
        .await
        .map_err(|e| match e {
            ApiLibError::Database(DbError::NotFound) => MessageDenied::NotFound,
            e => {
                eprintln!("Message owner error: {e}");
                MessageDenied::DatabaseError
            }
        })?;
    if is_admin || owner_id as u64 == user_id {
        Ok(())
    } else {
        Err(MessageDenied::Forbidden)
    }
}
