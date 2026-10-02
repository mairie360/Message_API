//! Chat membership check shared by every `/api/v1/{chat_id}/**` route.
//!
//! - Only the (not excluded) members of a chat may post in it and acknowledge its messages.
//! - Only the author of a message may edit it, nobody else (administrators included).
//! - Any member may leave a chat; only its creator (`conversations.created_by`, while still a
//!   member) or an administrator may add members or remove someone else.
//! - Administrators may read and moderate any chat: read it, list its members, manage them, delete
//!   any message (logged in `messaging_moderation_log`) and delete the chat.
//!
//! A caller who may not see the chat gets the same answer as for an unknown chat, so the routes
//! never reveal that a chat exists.
//!
//! Write routes check access with [`require_chat_access_in`] inside the transaction of their write
//! (MAIR-420): the check locks the chat and the caller's membership, so a member removed or a chat
//! deleted concurrently cannot slip between the check and the write.

use actix_web::web;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::database::chats::access::view::{ChatAccess, ChatAccessQueryView};
use crate::database::chats::message_owner::view::MessageOwnerQueryView;
use crate::endpoints::error::{classify, unexpected, DbFailure};

/// Body of the `403` answered to an administrator acting as a member of a chat they are not in.
pub const NOT_A_MEMBER_MESSAGE: &str =
    "Administrators may only read and moderate a chat they are not a member of.";
/// Body of the `403` answered to a member who may not manage the other members.
pub const NOT_A_MANAGER_MESSAGE: &str =
    "Only the creator of the chat or an administrator can manage its other members.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDenied {
    /// The chat does not exist, or the caller is neither a member nor an administrator.
    NotFound,
    DatabaseError,
}

/// The caller is an administrator who is not a member of the chat: they may read and moderate it,
/// not take part in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotAMember;

/// What the caller is in a chat they may see (see [`require_chat_access`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatRole {
    pub is_member: bool,
    pub is_creator: bool,
    pub is_admin: bool,
}

impl ChatRole {
    /// Posting and acknowledging reads are reserved to the members, administrators included.
    pub fn require_member(self) -> Result<(), NotAMember> {
        if self.is_member {
            Ok(())
        } else {
            Err(NotAMember)
        }
    }

    /// Adding members or removing someone else: the creator while still a member, or an
    /// administrator.
    pub fn can_manage_members(self) -> bool {
        self.is_admin || (self.is_member && self.is_creator)
    }
}

/// Checks that `user_id` may see chat `chat_id` (member or administrator) and returns their role.
/// For the read-only routes; write routes use [`require_chat_access_in`].
pub async fn require_chat_access(
    state: &web::Data<AppState>,
    chat_id: u64,
    user_id: u64,
) -> Result<ChatRole, AccessDenied> {
    let access: ChatAccess = state
        .get_smart_db()
        .fetch_one(&ChatAccessQueryView::new(chat_id, user_id))
        .await
        .map_err(|e| {
            unexpected("chat access", e);
            AccessDenied::DatabaseError
        })?;
    role_of(access)
}

/// Same as [`require_chat_access`] inside the transaction `tx` of a write: the chat and the
/// caller's membership stay locked until `tx` ends.
pub async fn require_chat_access_in(
    tx: &mut SmartTransaction,
    chat_id: u64,
    user_id: u64,
) -> Result<ChatRole, AccessDenied> {
    let access: ChatAccess = tx
        .fetch_one(&ChatAccessQueryView::locking(chat_id, user_id))
        .await
        .map_err(|e| {
            unexpected("chat access", e);
            AccessDenied::DatabaseError
        })?;
    role_of(access)
}

/// Opens the transaction of a write route.
pub async fn begin(state: &web::Data<AppState>) -> Result<SmartTransaction, AccessDenied> {
    state.get_smart_db().begin().await.map_err(|e| {
        unexpected("begin transaction", e);
        AccessDenied::DatabaseError
    })
}

/// Commits the transaction of a write route.
pub async fn commit(tx: SmartTransaction) -> Result<(), AccessDenied> {
    tx.commit().await.map_err(|e| {
        unexpected("commit transaction", e);
        AccessDenied::DatabaseError
    })
}

fn role_of(access: ChatAccess) -> Result<ChatRole, AccessDenied> {
    if !access.chat_exists || !(access.is_member || access.is_admin) {
        return Err(AccessDenied::NotFound);
    }
    Ok(ChatRole {
        is_member: access.is_member,
        is_creator: access.is_creator,
        is_admin: access.is_admin,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDenied {
    /// No message `message_id` in this chat.
    NotFound,
    /// The message belongs to someone else.
    Forbidden,
    DatabaseError,
}

/// Checks, inside the transaction `tx` of a write, that message `message_id` belongs to chat
/// `chat_id` and that `user_id` wrote it. With `may_moderate` (an administrator deleting a message)
/// the message of anyone is accepted.
pub async fn require_message_author_in(
    tx: &mut SmartTransaction,
    chat_id: u64,
    message_id: u64,
    user_id: u64,
    may_moderate: bool,
) -> Result<(), MessageDenied> {
    let owner_id: i32 = tx
        .fetch_scalar(&MessageOwnerQueryView::new(chat_id, message_id))
        .await
        .map_err(|e| match classify("message owner", e) {
            DbFailure::NotFound => MessageDenied::NotFound,
            _ => MessageDenied::DatabaseError,
        })?;
    if may_moderate || owner_id as u64 == user_id {
        Ok(())
    } else {
        Err(MessageDenied::Forbidden)
    }
}
