//! `/{chat_id}/messages/{message_id}/reactions/` (MAIR-1155): the reactions of a message.

pub mod delete;
pub mod doc;
pub mod post;

use mairie360_api_lib::smart_db::SmartTransaction;

use crate::database::chats::get_chat::view::ReactionRow;
use crate::database::chats::message_reactions::view::{
    MessageReactionsQueryView, MessageState, MessageStateQueryView, TouchMessageQueryView,
};
use crate::endpoints::error::{classify, unexpected, DbFailure};
use crate::endpoints::v1::id::access::{require_chat_access_in, AccessDenied};
use crate::endpoints::v1::id::get::view::ReactionView;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/reactions")
            .service(post::endpoint::add_reaction)
            .service(delete::endpoint::remove_reaction),
    );
}

/// Why a reaction may not be changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionDenied {
    /// Unknown chat, or the caller may not see it.
    UnknownChat,
    /// An administrator who is not a member: they do not take part.
    NotAMember,
    /// No such message in the chat, or it is deleted.
    UnknownMessage,
    /// The message is a system note.
    SystemNote,
    DatabaseError,
}

/// Checks, inside the transaction `tx`, that `user_id` is a member of chat `chat_id` and that
/// `message_id` is a live message of a user there, then marks the message as changed and locks it
/// (see [`TouchMessageQueryView`]).
pub async fn prepare_reaction(
    tx: &mut SmartTransaction,
    chat_id: u64,
    message_id: u64,
    user_id: u64,
) -> Result<(), ReactionDenied> {
    let role =
        require_chat_access_in(tx, chat_id, user_id)
            .await
            .map_err(|denied| match denied {
                AccessDenied::NotFound => ReactionDenied::UnknownChat,
                AccessDenied::DatabaseError => ReactionDenied::DatabaseError,
            })?;
    role.require_member()
        .map_err(|_| ReactionDenied::NotAMember)?;
    let state: MessageState = tx
        .fetch_one(&MessageStateQueryView::new(chat_id, message_id))
        .await
        .map_err(|e| match classify("message state", e) {
            DbFailure::NotFound => ReactionDenied::UnknownMessage,
            _ => ReactionDenied::DatabaseError,
        })?;
    if state.deleted {
        return Err(ReactionDenied::UnknownMessage);
    }
    if state.is_system() {
        return Err(ReactionDenied::SystemNote);
    }
    tx.fetch_scalar::<i64, _>(&TouchMessageQueryView::new(chat_id, message_id))
        .await
        .map_err(|e| match classify("touch message", e) {
            // Deleted between the state read and the lock.
            DbFailure::NotFound => ReactionDenied::UnknownMessage,
            _ => ReactionDenied::DatabaseError,
        })?;
    Ok(())
}

/// The reactions of `message_id` after the change, read in the same transaction.
pub async fn reactions_of(
    tx: &mut SmartTransaction,
    message_id: u64,
) -> Result<Vec<ReactionView>, ReactionDenied> {
    let rows: Vec<ReactionRow> = tx
        .fetch_all(&MessageReactionsQueryView::new(message_id))
        .await
        .map_err(|e| {
            unexpected("message reactions", e);
            ReactionDenied::DatabaseError
        })?;
    Ok(rows.into_iter().map(ReactionView::from).collect())
}
