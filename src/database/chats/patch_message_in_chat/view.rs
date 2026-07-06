use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct PatchMessageQueryView {
    message_id: u64,
    content: String,
}

impl PatchMessageQueryView {
    pub fn new(message_id: u64, content: &str) -> Self {
        Self {
            message_id,
            content: content.to_string(),
        }
    }

    pub fn message_id(&self) -> u64 {
        self.message_id
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Display for PatchMessageQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PatchMessageQueryView: message_id={}, content={}",
            self.message_id, self.content
        )
    }
}

impl DatabaseQueryView for PatchMessageQueryView {
    fn get_request(&self) -> String {
        "UPDATE messages SET content = $2 WHERE id = $1".to_string()
    }
}
