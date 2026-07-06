use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct ResetUnreadCountQueryView {
    chat_id: u64,
    user_id: u64,
}

impl ResetUnreadCountQueryView {
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

impl Display for ResetUnreadCountQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ResetUnreadCountQueryView: chat_id={} user_id={}",
            self.chat_id, self.user_id
        )
    }
}

impl DatabaseQueryView for ResetUnreadCountQueryView {
    fn get_request(&self) -> String {
        "UPDATE unread_counters SET unread_count = 0 WHERE conversation_id = $1 AND user_id = $2"
            .to_string()
    }
}
