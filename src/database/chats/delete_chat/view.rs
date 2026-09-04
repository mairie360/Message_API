use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteChatQueryView {
    params: Vec<QueryParam>,
}

impl DeleteChatQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(chat_id as i32)],
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl Display for DeleteChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeleteChatQueryView: chat_id={}", self.chat_id())
    }
}

impl ApiRequestDto for DeleteChatQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM conversations WHERE id = $1 RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
