use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct RemoveMemberFromChatQueryView {
    chat_id: u64,
    user_id: u64,
}

impl RemoveMemberFromChatQueryView {
    pub fn new(chat_id: u64, user_id: u64) -> Self {
        Self { chat_id, user_id }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

impl Display for RemoveMemberFromChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RemoveMemberFromChatQueryView: chat_id={} user_id={}",
            self.chat_id, self.user_id
        )
    }
}

impl DatabaseQueryView for RemoveMemberFromChatQueryView {
    fn get_request(&self) -> String {
        "DELETE FROM conversation_members WHERE conversation_id = $1 AND user_id = $2".to_string()
    }
}
