use crate::database::ids::id_to_sql;
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

/// What a global search result is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlobalSearchKind {
    /// A group chat whose title matches; `id` is reusable in `GET /api/v1/{chat_id}/`.
    Conversation,
    /// A message whose content matches; `parent_id` is its chat.
    Message,
}

/// One result of the global search.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct GlobalSearchItem {
    pub kind: GlobalSearchKind,
    /// Id of the chat or of the message.
    #[schema(example = 7)]
    pub id: i64,
    /// Displayed name of the chat (the message's chat for a message): the title of a group chat,
    /// the other participant's full name for a direct chat.
    #[schema(example = "Roads team")]
    pub title: String,
    /// First 120 characters of the message; `null` for a chat.
    #[schema(example = "The road marking of Lilac Street is planned for Thursday")]
    pub excerpt: Option<String>,
    /// Chat of a message; `null` for a chat.
    #[schema(example = json!(null))]
    pub parent_id: Option<i64>,
    /// Last activity of a chat, last change of a message (UTC, RFC 3339).
    #[schema(format = DateTime, example = "2026-11-20T09:30:00Z")]
    pub updated_at: String,
}

/// Chats and messages matching a global search (`GET /api/v1/search`), among the chats the
/// caller is a non-excluded member of: group chats whose title contains the term, and the live
/// messages of users (no tombstone, no system note) whose content contains it, through the
/// trigram indexes. The text starting with the term comes first, then the most recent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSearchQueryView {
    params: Vec<QueryParam>,
}

impl GlobalSearchQueryView {
    /// `pattern` is the term already escaped for `LIKE` (see `like_escape`), `limit` bounded by
    /// the endpoint.
    pub fn new(user_id: u64, pattern: &str, limit: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::Text(pattern.to_string()),
                QueryParam::I64(i64::try_from(limit).unwrap_or(i64::MAX)),
            ],
        }
    }
}

impl ApiRequestDto for GlobalSearchQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH mine AS MATERIALIZED ( \
             SELECT c.id, c.kind, c.last_activity_at, \
                    CASE WHEN c.kind = 'direct' \
                         THEN contact.first_name || ' ' || contact.last_name \
                         ELSE c.title END AS title, \
                    c.title AS group_title \
             FROM conversations c \
             JOIN conversation_members cm ON cm.conversation_id = c.id \
             LEFT JOIN users contact ON c.kind = 'direct' AND contact.id = \
                 CASE c.direct_user_low WHEN $1 THEN c.direct_user_high ELSE c.direct_user_low END \
             WHERE cm.user_id = $1 AND cm.is_excluded = FALSE \
         ), hits AS ( \
             SELECT 'conversation' AS kind, mine.id, mine.title, NULL::text AS excerpt, \
                    NULL::int AS parent_id, mine.last_activity_at AS changed_at, \
                    mine.group_title AS matched \
             FROM mine \
             WHERE mine.kind <> 'direct' AND mine.group_title ILIKE '%' || $2 || '%' \
             UNION ALL \
             SELECT 'message', m.id, COALESCE(mine.title, ''), left(m.content, 120), \
                    m.conversation_id, m.updated_at, m.content \
             FROM messages m JOIN mine ON mine.id = m.conversation_id \
             WHERE m.deleted_at IS NULL AND m.kind = 'user' \
               AND m.content ILIKE '%' || $2 || '%' \
         ) \
         SELECT jsonb_build_object( \
             'kind', h.kind, 'id', h.id, 'title', h.title, 'excerpt', h.excerpt, \
             'parent_id', h.parent_id, \
             'updated_at', to_char(h.changed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')) \
         FROM hits h \
         ORDER BY (h.matched ILIKE $2 || '%') DESC, h.changed_at DESC NULLS LAST, \
                  h.kind, h.id DESC \
         LIMIT $3"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

impl Display for GlobalSearchQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The search term is not logged.
        write!(f, "GlobalSearchQueryView: limit={:?}", self.params[2])
    }
}
