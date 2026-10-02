use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoveMemberFromChatQueryView {
    params: Vec<QueryParam>,
}

impl RemoveMemberFromChatQueryView {
    pub fn new(chat_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::I32(id_to_sql(user_id)),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl Display for RemoveMemberFromChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RemoveMemberFromChatQueryView: chat_id={} user_id={}",
            self.chat_id(),
            self.user_id()
        )
    }
}

impl ApiRequestDto for RemoveMemberFromChatQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM conversation_members WHERE conversation_id = $1 AND user_id = $2 RETURNING user_id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
