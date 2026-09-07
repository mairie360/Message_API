use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatQueryView {
    params: Vec<QueryParam>,
}

impl GetChatQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(chat_id as i32)],
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl Display for GetChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GetChatQueryView: chat_id={}", self.chat_id())
    }
}

impl ApiRequestDto for GetChatQueryView {
    fn query_sql(&self) -> &'static str {
        // `SmartDatabase`/`Database` décodent chaque ligne depuis une unique
        // colonne JSON : on sérialise donc la ligne avec `to_jsonb`.
        "SELECT to_jsonb(t) FROM (
            SELECT id, owner_id, content, created_at
            FROM messages
            WHERE conversation_id = $1
            ORDER BY created_at
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: i64,
    pub owner_id: i32,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
