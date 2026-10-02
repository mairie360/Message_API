use std::fmt::Display;

use crate::database::ids::{bigint_from_sql, bigint_to_sql, id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Posts a message, optionally as a reply to another message of the same chat
/// (`messages.reply_to_id`, whose composite foreign key rejects a message of another chat).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostMessageInChatQueryView {
    params: Vec<QueryParam>,
}

impl PostMessageInChatQueryView {
    pub fn new(chat_id: u64, sender: u64, message: &str) -> Self {
        Self::replying_to(chat_id, sender, message, None)
    }

    /// `reply_to` is the id of the quoted message (the API's `citation`).
    pub fn replying_to(chat_id: u64, sender: u64, message: &str, reply_to: Option<u64>) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::I32(id_to_sql(sender)),
                QueryParam::Text(message.to_string()),
                // No optional BIGINT in `QueryParam`: 0 (never a message id) stands for "none".
                QueryParam::I64(reply_to.map_or(0, bigint_to_sql)),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn sender(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }

    pub fn message(&self) -> &str {
        self.params[2].as_text()
    }

    pub fn reply_to(&self) -> Option<u64> {
        match self.params[3].as_i64() {
            0 => None,
            id => Some(bigint_from_sql(id)),
        }
    }
}

impl Display for PostMessageInChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PostMessageInChatQueryView: chat_id={} sender={} reply_to={:?} message={}",
            self.chat_id(),
            self.sender(),
            self.reply_to(),
            self.message()
        )
    }
}

impl ApiRequestDto for PostMessageInChatQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO messages (conversation_id, owner_id, content, reply_to_id) \
         VALUES ($1, $2, $3, NULLIF($4::bigint, 0)) RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
