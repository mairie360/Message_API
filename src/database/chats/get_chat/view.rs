use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// One page of the messages of a chat, newest first: the `limit` messages whose id is below
/// `before` (or the latest ones without `before`). Message ids grow in commit order within a chat
/// (Database `fn_before_message_insert`), so the id is a stable keyset cursor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatQueryView {
    params: Vec<QueryParam>,
}

impl GetChatQueryView {
    pub fn new(chat_id: u64, before: Option<u64>, limit: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(chat_id as i32),
                // No optional BIGINT in `QueryParam`: 0 (never a message id) means "no cursor".
                QueryParam::I64(before.map_or(0, |id| id as i64)),
                QueryParam::I32(limit as i32),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn before(&self) -> Option<u64> {
        match self.params[1].as_i64() {
            0 => None,
            id => Some(id as u64),
        }
    }

    pub fn limit(&self) -> u32 {
        self.params[2].as_i32() as u32
    }
}

impl Display for GetChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GetChatQueryView: chat_id={} before={:?} limit={}",
            self.chat_id(),
            self.before(),
            self.limit()
        )
    }
}

impl ApiRequestDto for GetChatQueryView {
    fn query_sql(&self) -> &'static str {
        // `SmartDatabase` decodes every row from a single JSON column, hence `to_jsonb`.
        "SELECT to_jsonb(t) FROM ( \
             SELECT id, owner_id, content, created_at, reply_to_id \
             FROM messages \
             WHERE conversation_id = $1 AND ($2::bigint = 0 OR id < $2::bigint) \
             ORDER BY id DESC \
             LIMIT $3 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: i64,
    /// `NULL` once the author's account is deleted (`ON DELETE SET NULL`).
    pub owner_id: Option<i32>,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The quoted message, `NULL` when none or once it is deleted.
    pub reply_to_id: Option<i64>,
}
