use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostMessageInChatQueryView {
    params: Vec<QueryParam>,
}

impl PostMessageInChatQueryView {
    pub fn new(chat_id: u64, sender: u64, message: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I32(chat_id as i32),
                QueryParam::I32(sender as i32),
                QueryParam::Text(message.to_string()),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn sender(&self) -> u64 {
        self.params[1].as_i32() as u64
    }

    pub fn message(&self) -> &str {
        self.params[2].as_text()
    }
}

impl Display for PostMessageInChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PostMessageInChatQueryView: chat_id={} sender={} message={}",
            self.chat_id(),
            self.sender(),
            self.message()
        )
    }
}

impl ApiRequestDto for PostMessageInChatQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO messages (conversation_id, owner_id, content) VALUES ($1, $2, $3) RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
