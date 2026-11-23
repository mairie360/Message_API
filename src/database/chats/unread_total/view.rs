use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Total of the unread messages of `user_id` over the chats of their list (MAIR-1106): the sum of
/// their `unread_counters` on the chats where they are a visible member (`is_excluded = FALSE`, the
/// same chats as `GET /api/v1/`), and how many of those chats have at least one. One aggregate row,
/// zeros when the user has no chat.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnreadTotalQueryView {
    params: Vec<QueryParam>,
}

impl UnreadTotalQueryView {
    pub fn new(user_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(id_to_sql(user_id))],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl Display for UnreadTotalQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnreadTotalQueryView: user_id={}", self.user_id())
    }
}

impl ApiRequestDto for UnreadTotalQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
            SELECT \
                COALESCE(SUM(uc.unread_count), 0)::int AS unread_count, \
                COUNT(*) FILTER (WHERE uc.unread_count > 0)::int AS unread_chats \
            FROM conversation_members cm \
            LEFT JOIN unread_counters uc \
                ON uc.conversation_id = cm.conversation_id AND uc.user_id = cm.user_id \
            WHERE cm.user_id = $1 AND cm.is_excluded = FALSE \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// The aggregate row of [`UnreadTotalQueryView`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnreadTotalQueryResultView {
    pub unread_count: i32,
    pub unread_chats: i32,
}
