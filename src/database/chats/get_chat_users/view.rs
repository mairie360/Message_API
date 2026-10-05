use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Ids of the (not excluded) members of chat `chat_id`, by increasing id.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatMembersQueryView {
    params: Vec<QueryParam>,
}

/// `LIMIT` of a page: `limit + 1` rows, the extra one only tells whether there is a next page.
fn page_limit(limit: u32) -> QueryParam {
    QueryParam::OptionI32(Some(i32::try_from(limit).unwrap_or(i32::MAX - 1) + 1))
}

fn page_offset(offset: u32) -> QueryParam {
    QueryParam::I32(i32::try_from(offset).unwrap_or(i32::MAX))
}

impl GetChatMembersQueryView {
    /// Every member: the SSE fan-out notifies all of them.
    pub fn new(chat_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                // LIMIT NULL: no limit.
                QueryParam::OptionI32(None),
                QueryParam::I32(0),
            ],
        }
    }

    /// One page for `GET /{chat_id}/users/`: `limit + 1` members from `offset` (see
    /// `endpoints::pagination::split_page`).
    pub fn page(chat_id: u64, limit: u32, offset: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                page_limit(limit),
                page_offset(offset),
            ],
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
         WHERE conversation_id = $1 AND is_excluded = FALSE \
         ORDER BY user_id LIMIT $2 OFFSET $3"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
