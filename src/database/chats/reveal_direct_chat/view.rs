use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Shows chat `chat_id` again to the participants who hid it, when it is a direct chat
/// (MAIR-478); no-op on a group chat. Run before the message is inserted, in the same
/// transaction, so the unread-counter trigger counts the message for them.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RevealDirectChatQueryView {
    params: Vec<QueryParam>,
}

impl RevealDirectChatQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(chat_id))],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl Display for RevealDirectChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RevealDirectChatQueryView: chat_id={}", self.chat_id())
    }
}

impl ApiRequestDto for RevealDirectChatQueryView {
    fn query_sql(&self) -> &'static str {
        "UPDATE conversation_members m SET is_excluded = FALSE \
         FROM conversations c \
         WHERE c.id = $1 AND c.kind = 'direct' AND m.conversation_id = c.id AND m.is_excluded"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
