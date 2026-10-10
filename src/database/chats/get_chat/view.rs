use std::fmt::Display;

use crate::database::ids::{bigint_from_sql, bigint_to_sql, id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// One page of the messages of a chat, newest first: the `limit` messages whose id is below
/// `before` (or the latest ones without `before`). Message ids grow in commit order within a chat
/// (Database `fn_before_message_insert`), so the id is a stable keyset cursor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatQueryView {
    params: Vec<QueryParam>,
}

impl GetChatQueryView {
    pub fn new(chat_id: u64, before: Option<u64>, limit: u32) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                // No optional BIGINT in `QueryParam`: 0 (never a message id) means "no cursor".
                QueryParam::I64(before.map_or(0, bigint_to_sql)),
                QueryParam::I32(i32::try_from(limit).unwrap_or(i32::MAX)),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn before(&self) -> Option<u64> {
        match self.params[1].as_i64() {
            0 => None,
            id => Some(bigint_from_sql(id)),
        }
    }

    pub fn limit(&self) -> u32 {
        u32::try_from(self.params[2].as_i32()).unwrap_or_default()
    }
}

impl Display for GetChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GetChatQueryView: chat_id={} before={:?} limit={}",
            self.chat_id(),
            self.before(),
            self.limit()
        )
    }
}

impl ApiRequestDto for GetChatQueryView {
    fn query_sql(&self) -> &'static str {
        // `SmartDatabase` decodes every row from a single JSON column, hence `to_jsonb`.
        // The quoted message is joined, not read apart: its author and an excerpt come with the
        // page even when it is older than the page (`reply_to_id`'s composite foreign key keeps it
        // in the same chat).
        "SELECT to_jsonb(t) FROM ( \
             SELECT m.id, m.owner_id, m.content, m.created_at, m.reply_to_id, \
                    q.owner_id AS reply_owner_id, left(q.content, 100) AS reply_excerpt \
             FROM messages m \
             LEFT JOIN messages q ON q.id = m.reply_to_id AND q.conversation_id = m.conversation_id \
             WHERE m.conversation_id = $1 AND ($2::bigint = 0 OR m.id < $2::bigint) \
             ORDER BY m.id DESC \
             LIMIT $3 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: i64,
    /// `NULL` once the author's account is deleted (`ON DELETE SET NULL`).
    pub owner_id: Option<i32>,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The quoted message, `NULL` when none or once it is deleted.
    pub reply_to_id: Option<i64>,
    /// Author of the quoted message: `NULL` when none is quoted or once its author's account is
    /// deleted.
    pub reply_owner_id: Option<i32>,
    /// First 100 characters of the quoted message, `NULL` when none is quoted.
    pub reply_excerpt: Option<String>,
}

/// Header of chat `chat_id` as seen by `user_id`: the same fields as a line of the chat list
/// (`GetChatsQueryResultView`), read for one chat. The name of a direct chat is the other
/// participant's; an administrator who is not a participant gets both names.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatHeaderQueryView {
    params: Vec<QueryParam>,
}

impl GetChatHeaderQueryView {
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

impl Display for GetChatHeaderQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GetChatHeaderQueryView: chat_id={} user_id={}",
            self.chat_id(),
            self.user_id()
        )
    }
}

impl ApiRequestDto for GetChatHeaderQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
             SELECT \
                 c.id, \
                 CASE \
                     WHEN c.kind <> 'direct' THEN c.title \
                     WHEN $2 IN (c.direct_user_low, c.direct_user_high) \
                         THEN other.first_name || ' ' || other.last_name \
                     ELSE low.first_name || ' ' || low.last_name \
                         || ' / ' || high.first_name || ' ' || high.last_name \
                 END AS title, \
                 c.kind, \
                 CASE $2 \
                     WHEN c.direct_user_low THEN c.direct_user_high \
                     WHEN c.direct_user_high THEN c.direct_user_low \
                 END AS contact_id, \
                 CASE WHEN c.kind = 'direct' THEN 2 ELSE \
                     (SELECT COUNT(*)::int FROM conversation_members m \
                      WHERE m.conversation_id = c.id AND m.is_excluded = FALSE) END AS member_count, \
                 COALESCE((SELECT uc.unread_count FROM unread_counters uc \
                           WHERE uc.conversation_id = c.id AND uc.user_id = $2), 0) AS unread_count \
             FROM conversations c \
             LEFT JOIN users low ON low.id = c.direct_user_low \
             LEFT JOIN users high ON high.id = c.direct_user_high \
             LEFT JOIN users other ON other.id = CASE c.direct_user_low \
                 WHEN $2 THEN c.direct_user_high ELSE c.direct_user_low END \
             WHERE c.id = $1 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
