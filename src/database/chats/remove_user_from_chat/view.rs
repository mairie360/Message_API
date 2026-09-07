use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoveMemberFromChatQueryView {
    params: Vec<QueryParam>,
}

impl RemoveMemberFromChatQueryView {
    pub fn new(chat_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(chat_id as i32),
                QueryParam::I32(user_id as i32),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn user_id(&self) -> u64 {
        self.params[1].as_i32() as u64
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
