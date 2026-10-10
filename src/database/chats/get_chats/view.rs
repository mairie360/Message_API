use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Chats of `user_id` with their displayed name (the title of a group chat, the other participant's
/// full name for a direct chat), kind, contact of a direct chat, member count and unread counter,
/// newest first. Optionally filtered by a search term.
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

/// `ILIKE` pattern matching `term` anywhere: `%`, `_` and `\` of the term are escaped so that they
/// only match themselves. The empty string (no search) is passed as is and disables the filter.
fn search_pattern(term: Option<&str>) -> QueryParam {
    let term = term.map(str::trim).unwrap_or_default();
    if term.is_empty() {
        return QueryParam::Text(String::new());
    }
    let mut pattern = String::with_capacity(term.len() + 2);
    pattern.push('%');
    for c in term.chars() {
        if matches!(c, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');
    QueryParam::Text(pattern)
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
                search_pattern(None),
            ],
        }
    }

    /// One page for `GET /api/v1/`: `limit + 1` chats from `offset` (see
    /// `endpoints::pagination::split_page`), among the chats matching `search` when given: the
    /// name of the chat, or the name of one of its members other than the caller (the other
    /// participant of a direct chat).
    pub fn page(user_id: u64, limit: u32, offset: u32, search: Option<&str>) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                page_limit(limit),
                page_offset(offset),
                search_pattern(search),
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
        // One statement per page, whatever the number of chats: the direct contact's name and the
        // member count are joined / counted in the same query. `$4` is the `ILIKE` pattern of the
        // search ('' = none); a group's members other than the caller are only probed for the
        // chats of the page candidates, through `conversation_members`' primary key.
        "SELECT to_jsonb(t) FROM (
            SELECT
                c.id,
                CASE WHEN c.kind = 'direct'
                    THEN contact.first_name || ' ' || contact.last_name
                    ELSE c.title
                END AS title,
                c.kind,
                CASE c.direct_user_low
                    WHEN $1 THEN c.direct_user_high
                    ELSE c.direct_user_low
                END AS contact_id,
                CASE WHEN c.kind = 'direct' THEN 2 ELSE (
                    SELECT COUNT(*)::int
                    FROM conversation_members m
                    WHERE m.conversation_id = c.id AND m.is_excluded = FALSE
                ) END AS member_count,
                COALESCE(uc.unread_count, 0) AS unread_count
            FROM conversations c
            INNER JOIN conversation_members cm ON c.id = cm.conversation_id
            LEFT JOIN unread_counters uc
                ON c.id = uc.conversation_id AND uc.user_id = cm.user_id
            LEFT JOIN users contact ON c.kind = 'direct' AND contact.id = (
                CASE c.direct_user_low
                    WHEN $1 THEN c.direct_user_high
                    ELSE c.direct_user_low
                END
            )
            WHERE cm.user_id = $1 AND cm.is_excluded = FALSE
              AND (
                $4 = ''
                OR c.title ILIKE $4
                OR contact.first_name ILIKE $4
                OR contact.last_name ILIKE $4
                OR (contact.first_name || ' ' || contact.last_name) ILIKE $4
                OR (contact.last_name || ' ' || contact.first_name) ILIKE $4
                OR (c.kind <> 'direct' AND EXISTS (
                    SELECT 1
                    FROM conversation_members m2
                    INNER JOIN users u ON u.id = m2.user_id
                    WHERE m2.conversation_id = c.id
                      AND m2.is_excluded = FALSE
                      AND m2.user_id <> $1
                      AND (
                        u.first_name ILIKE $4
                        OR u.last_name ILIKE $4
                        OR (u.first_name || ' ' || u.last_name) ILIKE $4
                        OR (u.last_name || ' ' || u.first_name) ILIKE $4
                      )
                ))
              )
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
    /// Title of a group chat (`NULL` when untitled), full name of the other participant of a
    /// direct chat (`NULL` when that account is gone).
    pub title: Option<String>,
    /// `conversations.kind`: `direct` or `group`.
    pub kind: String,
    /// The other participant of a direct chat, `None` for a group chat.
    pub contact_id: Option<i32>,
    /// Members that did not leave the chat; a direct chat always has its two participants.
    pub member_count: i32,
    pub unread_count: i32,
}
