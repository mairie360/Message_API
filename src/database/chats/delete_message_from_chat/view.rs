use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct DeleteMessageQueryView {
    message_id: u64,
}

impl DeleteMessageQueryView {
    pub fn new(message_id: u64) -> Self {
        Self { message_id }
    }

    pub fn message_id(&self) -> u64 {
        self.message_id
    }
}

impl Display for DeleteMessageQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeleteMessageQueryView: message_id={}", self.message_id)
    }
}

impl DatabaseQueryView for DeleteMessageQueryView {
    fn get_request(&self) -> String {
        "DELETE FROM messages WHERE id = $1".to_string()
    }
}
