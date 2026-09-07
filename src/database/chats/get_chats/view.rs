use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatsQueryView {
    params: Vec<QueryParam>,
}

impl GetChatsQueryView {
    pub fn new(user_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(user_id as i32)],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }
}

impl Display for GetChatsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GetChatsQueryView: user_id={}", self.user_id())
    }
}

impl ApiRequestDto for GetChatsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM (
            SELECT
                c.id,
                c.title,
                COALESCE(uc.unread_count, 0) AS unread_count
            FROM conversations c
            INNER JOIN conversation_members cm ON c.id = cm.conversation_id
            LEFT JOIN unread_counters uc
                ON c.id = uc.conversation_id AND uc.user_id = cm.user_id
            WHERE cm.user_id = $1 AND cm.is_excluded = FALSE
            ORDER BY c.created_at DESC
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatsQueryResultView {
    pub id: i32,
    pub title: Option<String>,
    pub unread_count: i32,
}
