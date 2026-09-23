use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Deletes chat `chat_id` once it has no (not excluded) member left; no-op otherwise. A chat is
/// never deleted directly by its members: it goes away with its last member.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteEmptyChatQueryView {
    params: Vec<QueryParam>,
}

impl DeleteEmptyChatQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(chat_id as i32)],
        }
    }
}

impl Display for DeleteEmptyChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeleteEmptyChatQueryView: {:?}", self.params)
    }
}

impl ApiRequestDto for DeleteEmptyChatQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM conversations c WHERE c.id = $1 AND NOT EXISTS ( \
            SELECT 1 FROM conversation_members m \
            WHERE m.conversation_id = $1 AND m.is_excluded = FALSE)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
