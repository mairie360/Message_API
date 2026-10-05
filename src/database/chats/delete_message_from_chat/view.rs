use std::fmt::Display;

use crate::database::ids::{bigint_from_sql, bigint_to_sql, id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Deletes message `message_id` of chat `chat_id` on behalf of `performed_by`. When the message
/// was written by someone else (moderation by an administrator), the same statement appends a
/// `DELETE_MESSAGE` row with a snapshot of the content to `messaging_moderation_log`.
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
        "WITH deleted AS ( \
             DELETE FROM messages WHERE id = $2 AND conversation_id = $1 \
             RETURNING id, conversation_id, owner_id, content \
         ), logged AS ( \
             INSERT INTO messaging_moderation_log \
                 (action, conversation_id, message_id, target_user_id, content, performed_by) \
             SELECT 'DELETE_MESSAGE', conversation_id, id, owner_id, content, $3 \
             FROM deleted WHERE owner_id IS DISTINCT FROM $3 \
         ) \
         SELECT id FROM deleted"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
