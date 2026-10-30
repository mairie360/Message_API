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
        //
        // Tombstones (MAIR-524, `deleted_at`) are not part of the history, and the excerpt of a
        // deleted quoted message is hidden (`citation` reads `NULL` as before the soft delete).
        "SELECT to_jsonb(t) FROM ( \
             SELECT m.id, m.owner_id, m.content, m.created_at, \
                    CASE WHEN q.id IS NOT NULL THEN m.reply_to_id END AS reply_to_id, \
                    q.owner_id AS reply_owner_id, left(q.content, 100) AS reply_excerpt, \
                    m.updated_at, m.edited_at, FALSE AS deleted \
             FROM messages m \
             LEFT JOIN messages q ON q.id = m.reply_to_id AND q.conversation_id = m.conversation_id \
                 AND q.deleted_at IS NULL \
             WHERE m.conversation_id = $1 AND m.deleted_at IS NULL \
               AND ($2::bigint = 0 OR m.id < $2::bigint) \
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
    /// Last change of the message (posted, edited, deleted): the cursor of the reads since.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Last edit of the content, `NULL` when never edited.
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    /// A tombstone: the message was deleted, its content is empty.
    pub deleted: bool,
}

/// The messages of chat `chat_id` changed since the cursor `(since, since_id)` (MAIR-524): posted,
/// edited or deleted (tombstones, with an empty content), in change order `(updated_at, id)`,
/// at most `limit` rows. Within a chat, Database `fn_track_message_change` stamps `updated_at`
/// under the conversation lock, so a change committed later always sorts after one already read:
/// passing back the cursor of the last row read never skips a change.
/// Served by `idx_messages_conversation_updated (conversation_id, updated_at, id)`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetChatChangesQueryView {
    params: Vec<QueryParam>,
}

impl GetChatChangesQueryView {
    pub fn new(
        chat_id: u64,
        since: chrono::DateTime<chrono::Utc>,
        since_id: u64,
        limit: u32,
    ) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::DateTime(since),
                QueryParam::I64(bigint_to_sql(since_id)),
                QueryParam::I32(i32::try_from(limit).unwrap_or(i32::MAX)),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn limit(&self) -> u32 {
        u32::try_from(self.params[3].as_i32()).unwrap_or_default()
    }
}

impl Display for GetChatChangesQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GetChatChangesQueryView: chat_id={} cursor={:?}/{:?} limit={}",
            self.chat_id(),
            self.params[1],
            self.params[2],
            self.limit()
        )
    }
}

impl ApiRequestDto for GetChatChangesQueryView {
    fn query_sql(&self) -> &'static str {
        // A tombstone carries no content nor quote; a live message quoting a deleted one loses
        // its quote like in the history.
        "SELECT to_jsonb(t) FROM ( \
             SELECT m.id, m.owner_id, m.content, m.created_at, \
                    CASE WHEN q.id IS NOT NULL THEN m.reply_to_id END AS reply_to_id, \
                    q.owner_id AS reply_owner_id, left(q.content, 100) AS reply_excerpt, \
                    m.updated_at, m.edited_at, m.deleted_at IS NOT NULL AS deleted \
             FROM messages m \
             LEFT JOIN messages q ON q.id = m.reply_to_id AND q.conversation_id = m.conversation_id \
                 AND q.deleted_at IS NULL AND m.deleted_at IS NULL \
             WHERE m.conversation_id = $1 \
               AND (m.updated_at, m.id) > ($2::timestamptz, $3::bigint) \
             ORDER BY m.updated_at, m.id \
             LIMIT $4 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Header of chat `chat_id` as seen by `user_id`: the same fields as a line of the chat list
/// (`GetChatsQueryResultView`), read for one chat, plus the cursor of its latest message change
/// ([`ChatHeader`]). The name of a direct chat is the other participant's; an administrator who is
/// not a participant gets both names.
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
                           WHERE uc.conversation_id = c.id AND uc.user_id = $2), 0) AS unread_count, \
                 c.last_activity_at, \
                 COALESCE(latest.updated_at, c.last_activity_at) AS latest_change_at, \
                 COALESCE(latest.id, 0) AS latest_change_id \
             FROM conversations c \
             LEFT JOIN LATERAL ( \
                 SELECT lm.updated_at, lm.id FROM messages lm WHERE lm.conversation_id = c.id \
                 ORDER BY lm.updated_at DESC, lm.id DESC LIMIT 1 \
             ) latest ON TRUE \
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

/// Row of [`GetChatHeaderQueryView`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatHeader {
    #[serde(flatten)]
    pub chat: crate::database::chats::get_chats::view::GetChatsQueryResultView,
    /// `updated_at` of the latest change of a message of the chat, `last_activity_at` of the chat
    /// when it has no message.
    pub latest_change_at: chrono::DateTime<chrono::Utc>,
    /// Id of that message, `0` when the chat has no message.
    pub latest_change_id: i64,
}
