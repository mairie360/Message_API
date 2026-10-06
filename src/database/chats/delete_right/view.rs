use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Whether `check_access` grants `user_id` the `delete` right on conversation `chat_id`: the
/// global `delete_all` permission of one of their roles, or an individual or group ACL on this
/// conversation. The decision is recorded in `access_logs` by `check_access` itself.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatDeleteRightQueryView {
    params: Vec<QueryParam>,
}

impl ChatDeleteRightQueryView {
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

impl Display for ChatDeleteRightQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChatDeleteRightQueryView: chat_id={} user_id={}",
            self.chat_id(),
            self.user_id()
        )
    }
}

impl ApiRequestDto for ChatDeleteRightQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT check_access($2, 'conversations', 'delete', $1) = 1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
