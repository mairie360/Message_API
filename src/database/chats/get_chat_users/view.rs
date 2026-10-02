use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatMembersQueryView {
    params: Vec<QueryParam>,
}

impl GetChatMembersQueryView {
    pub fn new(chat_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(chat_id))],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl Display for GetChatMembersQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GetChatMembersQueryView: chat_id={}", self.chat_id())
    }
}

impl ApiRequestDto for GetChatMembersQueryView {
    fn query_sql(&self) -> &'static str {
        // Excluded members no longer see the chat: they are not notified either.
        "SELECT to_jsonb(user_id) FROM conversation_members \
         WHERE conversation_id = $1 AND is_excluded = FALSE"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
