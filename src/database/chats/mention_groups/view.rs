//! Groups that may be mentioned in a chat (MAIR-1155).

use std::fmt::Display;

use crate::database::chats::get_chats::view::search_pattern;
use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Most groups suggested at once.
pub const MAX_MENTION_GROUPS: i32 = 20;

/// The groups at least one non-excluded member of chat `chat_id` belongs to, whose name matches
/// `term` (`ILIKE '%term%'`, every group without a term), by name: at most
/// [`MAX_MENTION_GROUPS`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MentionGroupsQueryView {
    params: Vec<QueryParam>,
}

impl MentionGroupsQueryView {
    pub fn new(chat_id: u64, term: Option<&str>) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                search_pattern(term),
                QueryParam::I32(MAX_MENTION_GROUPS),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl Display for MentionGroupsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MentionGroupsQueryView: chat_id={}", self.chat_id())
    }
}

impl ApiRequestDto for MentionGroupsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
             SELECT g.id, g.name FROM groups g \
             WHERE ($2 = '' OR g.name ILIKE $2) \
               AND EXISTS ( \
                   SELECT 1 FROM group_members gm \
                   JOIN conversation_members cm ON cm.user_id = gm.user_id \
                       AND cm.conversation_id = $1 AND cm.is_excluded = FALSE \
                   WHERE gm.group_id = g.id) \
             ORDER BY g.name, g.id \
             LIMIT $3 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Row of [`MentionGroupsQueryView`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MentionGroupRow {
    pub id: i32,
    pub name: String,
}
