use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Author of message `message_id` of chat `chat_id` (`0` once the author's account is gone).
/// No row (`DbError::NotFound`) when the message does not belong to that chat.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageOwnerQueryView {
    params: Vec<QueryParam>,
}

impl MessageOwnerQueryView {
    pub fn new(chat_id: u64, message_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I64(message_id as i64),
                QueryParam::I32(chat_id as i32),
            ],
        }
    }
}

impl Display for MessageOwnerQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MessageOwnerQueryView: {:?}", self.params)
    }
}

impl ApiRequestDto for MessageOwnerQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT COALESCE(owner_id, 0) FROM messages WHERE id = $1 AND conversation_id = $2"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
