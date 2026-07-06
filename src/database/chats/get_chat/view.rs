use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct GetChatQueryView {
    chat_id: u64,
}

impl GetChatQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self { chat_id }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }
}

impl Display for GetChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GetChatQueryView: chat_id={}", self.chat_id)
    }
}

impl DatabaseQueryView for GetChatQueryView {
    fn get_request(&self) -> String {
        "SELECT id, owner_id, content, created_at from messages WHERE conversation_id = $1"
            .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Message {
    pub id: i64,
    pub owner_id: i32,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
