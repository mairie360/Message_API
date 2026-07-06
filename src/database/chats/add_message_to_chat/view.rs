use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct PostMessageInChatQueryView {
    chat_id: u64,
    sender: u64,
    message: String,
}

impl PostMessageInChatQueryView {
    pub fn new(chat_id: u64, sender: u64, message: &str) -> Self {
        Self {
            chat_id,
            sender,
            message: message.to_string(),
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }

    pub fn sender(&self) -> u64 {
        self.sender
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for PostMessageInChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PostMessageInChatQueryView: chat_id={} sender={} message={}",
            self.chat_id, self.sender, self.message
        )
    }
}

impl DatabaseQueryView for PostMessageInChatQueryView {
    fn get_request(&self) -> String {
        "INSERT INTO messages (conversation_id, owner_id, content) VALUES ($1, $2, $3) RETURNING id"
            .to_string()
    }
}
