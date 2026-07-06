use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct GetChatMembersQueryView {
    chat_id: u64,
}

impl GetChatMembersQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self { chat_id }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }
}

impl Display for GetChatMembersQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GetChatMembersQueryView: chat_id={}", self.chat_id)
    }
}

impl DatabaseQueryView for GetChatMembersQueryView {
    fn get_request(&self) -> String {
        "SELECT user_id from conversation_members WHERE conversation_id = $1".to_string()
    }
}
