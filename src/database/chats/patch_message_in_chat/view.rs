use std::fmt::Display;

use crate::database::ids::{bigint_from_sql, bigint_to_sql, id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Replaces the content of message `message_id` of chat `chat_id`. No row (`DbError::NotFound`)
/// when the message does not belong to that chat or is deleted (tombstone).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatchMessageQueryView {
    params: Vec<QueryParam>,
}

impl PatchMessageQueryView {
    pub fn new(chat_id: u64, message_id: u64, content: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I64(bigint_to_sql(message_id)),
                QueryParam::Text(content.to_string()),
                QueryParam::I32(id_to_sql(chat_id)),
                // No optional boolean in `QueryParam`: -1 keeps `mentions_everyone`.
                QueryParam::I32(-1),
            ],
        }
    }

    /// Replaces `mentions_everyone` too (MAIR-1155); `None` keeps it.
    pub fn mentioning_everyone(mut self, everyone: Option<bool>) -> Self {
        self.params[3] = QueryParam::I32(everyone.map_or(-1, i32::from));
        self
    }

    pub fn mentions_everyone(&self) -> Option<bool> {
        match self.params[3].as_i32() {
            -1 => None,
            value => Some(value == 1),
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[2].as_i32())
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[0].as_i64())
    }

    pub fn content(&self) -> &str {
        self.params[1].as_text()
    }
}

impl Display for PatchMessageQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PatchMessageQueryView: message_id={}, chat_id={}",
            self.message_id(),
            self.chat_id()
        )
    }
}

impl ApiRequestDto for PatchMessageQueryView {
    fn query_sql(&self) -> &'static str {
        // A tombstone is never edited back (MAIR-524), a system note never edited (MAIR-1155).
        "UPDATE messages SET content = $2, \
             mentions_everyone = CASE $4 WHEN -1 THEN mentions_everyone ELSE $4 = 1 END \
         WHERE id = $1 AND conversation_id = $3 AND deleted_at IS NULL AND kind = 'user' \
         RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
