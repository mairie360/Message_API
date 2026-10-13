use std::fmt::Display;

use crate::database::ids::{bigint_from_sql, bigint_to_sql, id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Deletes message `message_id` of chat `chat_id` on behalf of `performed_by`: it becomes a
/// tombstone (`deleted_at`, empty content), and the unread counters that counted it are lowered.
/// When the message was written by someone else (moderation by an administrator), the same
/// statement appends a `DELETE_MESSAGE` row with a snapshot of the content to
/// `messaging_moderation_log`. No row (`DbError::NotFound`) when the message is not in that chat
/// or already deleted.
///
/// Run it after [`LockChatMessagesQueryView`] in the same transaction: the counters are then
/// adjusted on a snapshot no read acknowledgement of the chat can change concurrently.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteMessageQueryView {
    params: Vec<QueryParam>,
}

impl DeleteMessageQueryView {
    pub fn new(chat_id: u64, message_id: u64, performed_by: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::I64(bigint_to_sql(message_id)),
                QueryParam::I32(id_to_sql(performed_by)),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[1].as_i64())
    }

    pub fn performed_by(&self) -> u64 {
        id_from_sql(self.params[2].as_i32())
    }
}

impl Display for DeleteMessageQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DeleteMessageQueryView: chat_id={} message_id={} performed_by={}",
            self.chat_id(),
            self.message_id(),
            self.performed_by()
        )
    }
}

impl ApiRequestDto for DeleteMessageQueryView {
    fn query_sql(&self) -> &'static str {
        // Soft delete (MAIR-524): the message stays as a tombstone so that the readers since a
        // cursor learn about it; Database `fn_track_message_change` empties its content, hence
        // the content logged is read before the update (`target`). The unread counters that
        // still count it are lowered: every recipient but the author whose read cursor is before
        // the message and who was already a member when it was posted.
        "WITH target AS ( \
             SELECT id, conversation_id, owner_id, content, created_at FROM messages \
             WHERE id = $2 AND conversation_id = $1 AND deleted_at IS NULL \
         ), deleted AS ( \
             UPDATE messages m SET deleted_at = now() FROM target t \
             WHERE m.id = t.id AND m.deleted_at IS NULL \
             RETURNING m.id, m.conversation_id, t.owner_id, t.content, t.created_at \
         ), logged AS ( \
             INSERT INTO messaging_moderation_log \
                 (action, conversation_id, message_id, target_user_id, content, performed_by) \
             SELECT 'DELETE_MESSAGE', conversation_id, id, owner_id, content, $3 \
             FROM deleted WHERE owner_id IS DISTINCT FROM $3 \
         ), unread AS ( \
             UPDATE unread_counters uc SET unread_count = uc.unread_count - 1 \
             FROM deleted d \
             WHERE uc.conversation_id = d.conversation_id \
               AND uc.user_id IS DISTINCT FROM d.owner_id \
               AND NOT EXISTS ( \
                   SELECT 1 FROM conversation_read_cursors rc \
                   WHERE rc.conversation_id = d.conversation_id AND rc.user_id = uc.user_id \
                     AND rc.last_read_message_id >= d.id) \
               AND NOT EXISTS ( \
                   SELECT 1 FROM conversation_members cm \
                   WHERE cm.conversation_id = d.conversation_id AND cm.user_id = uc.user_id \
                     AND cm.joined_at > d.created_at) \
         ) \
         SELECT id FROM deleted"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Takes the conversation lock of Database `fn_before_message_insert` / `fn_acknowledge_read`
/// (`pg_advisory_xact_lock(360, chat)`) until the end of the transaction: the next statement sees
/// every send and acknowledgement of the chat committed before, and none can run meanwhile.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LockChatMessagesQueryView {
    params: Vec<QueryParam>,
}

impl LockChatMessagesQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(chat_id))],
        }
    }
}

impl Display for LockChatMessagesQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LockChatMessagesQueryView: {:?}", self.params)
    }
}

impl ApiRequestDto for LockChatMessagesQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT pg_advisory_xact_lock(360, $1)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
