pub mod doc;
pub mod get;
pub mod id;
pub mod post;

use mairie360_api_lib::smart_db::SmartTransaction;

use crate::database::chats::get_chat::view::{GetMessageQueryView, Message};
use crate::database::chats::system_notes::view::{PostSystemNoteQueryView, SystemAction};
use crate::database::ids::bigint_from_sql;
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::id::get::view::MessageView;

/// The database failed while writing a system note (already logged): answer `500`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemNoteFailed;

/// Writes, inside the transaction of a member change, the system note recording it (MAIR-1155)
/// and reads it back as the history shows it. `None` when nobody sees the chat any more (its last
/// member left).
pub async fn post_system_note(
    tx: &mut SmartTransaction,
    chat_id: u64,
    actor_id: u64,
    action: SystemAction,
    user_ids: &[u64],
) -> Result<Option<MessageView>, SystemNoteFailed> {
    let ids: Vec<i64> = tx
        .fetch_all(&PostSystemNoteQueryView::new(
            chat_id, actor_id, action, user_ids,
        ))
        .await
        .map_err(|e| {
            unexpected("post system note", e);
            SystemNoteFailed
        })?;
    let Some(id) = ids.first() else {
        return Ok(None);
    };
    let message: Message = tx
        .fetch_one(&GetMessageQueryView::new(chat_id, bigint_from_sql(*id)))
        .await
        .map_err(|e| {
            unexpected("read system note", e);
            SystemNoteFailed
        })?;
    Ok(Some(MessageView::from(message)))
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/users")
            .service(get::endpoint::get_chat_users)
            .service(post::endpoint::add_users_to_chat)
            .configure(id::config),
    );
}
