use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteMessageQueryView {
    params: Vec<QueryParam>,
}

impl DeleteMessageQueryView {
    pub fn new(message_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(message_id as i32)],
        }
    }

    pub fn message_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl Display for DeleteMessageQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DeleteMessageQueryView: message_id={}",
            self.message_id()
        )
    }
}

impl ApiRequestDto for DeleteMessageQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM messages WHERE id = $1 RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
