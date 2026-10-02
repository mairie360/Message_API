use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Chats of `user_id` with their unread counter, newest first.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatsQueryView {
    params: Vec<QueryParam>,
}

/// `LIMIT` of a page: `limit + 1` rows, the extra one only tells whether there is a next page.
fn page_limit(limit: u32) -> QueryParam {
    QueryParam::OptionI32(Some(i32::try_from(limit).unwrap_or(i32::MAX - 1) + 1))
}

fn page_offset(offset: u32) -> QueryParam {
    QueryParam::I32(i32::try_from(offset).unwrap_or(i32::MAX))
}

impl GetChatsQueryView {
    /// Every chat of the user.
    pub fn new(user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                // LIMIT NULL: no limit.
                QueryParam::OptionI32(None),
                QueryParam::I32(0),
            ],
        }
    }

    /// One page for `GET /api/v1/`: `limit + 1` chats from `offset` (see
    /// `endpoints::pagination::split_page`).
    pub fn page(user_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                page_limit(limit),
                page_offset(offset),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
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
            ORDER BY c.created_at DESC, c.id DESC
            LIMIT $2 OFFSET $3
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
