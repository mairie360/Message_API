//! Search inside one chat (MAIR-1155): its messages or its members.

use std::fmt::Display;

use crate::database::chats::get_chats::view::search_pattern;
use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// What [`SearchChatQueryView`] looks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SearchTarget {
    /// The live messages of users (no tombstone, no system note) whose content matches, newest
    /// first.
    Messages,
    /// The non-excluded members whose first name, last name or full name (both orders) matches,
    /// by last name, first name and id.
    People,
}

/// One page of the matches of `term` in chat `chat_id` (`ILIKE '%term%'`, `%` `_` `\` escaped)
/// and their total, in one row: `{"total": n, "items": [...]}`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchChatQueryView {
    target: SearchTarget,
    params: Vec<QueryParam>,
}

impl SearchChatQueryView {
    pub fn new(target: SearchTarget, chat_id: u64, term: &str, limit: u32, offset: u32) -> Self {
        Self {
            target,
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                search_pattern(Some(term)),
                QueryParam::I32(i32::try_from(limit).unwrap_or(i32::MAX)),
                QueryParam::I32(i32::try_from(offset).unwrap_or(i32::MAX)),
            ],
        }
    }

    pub fn target(&self) -> SearchTarget {
        self.target
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }
}

impl Display for SearchChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The search term is not logged (MAIR-290).
        write!(
            f,
            "SearchChatQueryView: target={:?} chat_id={}",
            self.target,
            self.chat_id()
        )
    }
}

impl ApiRequestDto for SearchChatQueryView {
    fn query_sql(&self) -> &'static str {
        match self.target {
            SearchTarget::Messages => {
                "WITH matches AS ( \
                     SELECT m.id, m.owner_id AS sender_id, m.created_at, m.content \
                     FROM messages m \
                     WHERE m.conversation_id = $1 AND m.deleted_at IS NULL AND m.kind = 'user' \
                       AND m.content ILIKE $2 \
                 ) \
                 SELECT jsonb_build_object( \
                     'total', (SELECT COUNT(*)::int FROM matches), \
                     'items', COALESCE((SELECT jsonb_agg(to_jsonb(p) ORDER BY p.id DESC) \
                                        FROM (SELECT * FROM matches ORDER BY id DESC \
                                              LIMIT $3 OFFSET $4) p), '[]'::jsonb))"
            }
            SearchTarget::People => {
                "WITH matches AS ( \
                     SELECT u.id, u.first_name, u.last_name \
                     FROM conversation_members cm JOIN users u ON u.id = cm.user_id \
                     WHERE cm.conversation_id = $1 AND cm.is_excluded = FALSE \
                       AND (u.first_name ILIKE $2 OR u.last_name ILIKE $2 \
                            OR (u.first_name || ' ' || u.last_name) ILIKE $2 \
                            OR (u.last_name || ' ' || u.first_name) ILIKE $2) \
                 ) \
                 SELECT jsonb_build_object( \
                     'total', (SELECT COUNT(*)::int FROM matches), \
                     'items', COALESCE((SELECT jsonb_agg(to_jsonb(p) \
                                            ORDER BY p.last_name, p.first_name, p.id) \
                                        FROM (SELECT * FROM matches \
                                              ORDER BY last_name, first_name, id \
                                              LIMIT $3 OFFSET $4) p), '[]'::jsonb))"
            }
        }
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Row of [`SearchChatQueryView`].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SearchPage<T> {
    pub total: i32,
    pub items: Vec<T>,
}

/// A message matching the search.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MessageMatch {
    pub id: i64,
    pub sender_id: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content: String,
}

/// A member matching the search.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersonMatch {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
}
