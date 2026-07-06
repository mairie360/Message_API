use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct DeleteChatQueryView {
    chat_id: u64,
}

impl DeleteChatQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self { chat_id }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }
}

impl Display for DeleteChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeleteChatQueryView: chat_id={}", self.chat_id)
    }
}

impl DatabaseQueryView for DeleteChatQueryView {
    fn get_request(&self) -> String {
        "DELETE FROM conversations WHERE id = $1".to_string()
    }
}
