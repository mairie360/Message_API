//! Reactions of a message (MAIR-1155): one row per (message, user, emoji).

use std::fmt::Display;

use crate::database::ids::{bigint_from_sql, bigint_to_sql, id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Kind and state of message `message_id` of chat `chat_id`; no row when it is not in that chat.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageStateQueryView {
    params: Vec<QueryParam>,
}

impl MessageStateQueryView {
    pub fn new(chat_id: u64, message_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::I64(bigint_to_sql(message_id)),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[1].as_i64())
    }
}

impl Display for MessageStateQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MessageStateQueryView: chat_id={} message_id={}",
            self.chat_id(),
            self.message_id()
        )
    }
}

impl ApiRequestDto for MessageStateQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(t) FROM ( \
             SELECT kind, deleted_at IS NOT NULL AS deleted FROM messages \
             WHERE conversation_id = $1 AND id = $2 \
         ) t"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Row of [`MessageStateQueryView`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MessageState {
    /// `user` or `system`.
    pub kind: String,
    /// A tombstone.
    pub deleted: bool,
}

impl MessageState {
    pub fn is_system(&self) -> bool {
        self.kind == "system"
    }
}

/// Marks message `message_id` of chat `chat_id` as changed (Database `fn_track_message_change`
/// moves its `updated_at`, so the incremental reads return it with its new reactions) and locks
/// its row until the end of the transaction: a concurrent deletion waits for the reaction, then
/// purges it. No row when the message is not a live message of a user of that chat.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TouchMessageQueryView {
    params: Vec<QueryParam>,
}

impl TouchMessageQueryView {
    pub fn new(chat_id: u64, message_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::I64(bigint_to_sql(message_id)),
            ],
        }
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[1].as_i64())
    }
}

impl Display for TouchMessageQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TouchMessageQueryView: message_id={}", self.message_id())
    }
}

impl ApiRequestDto for TouchMessageQueryView {
    fn query_sql(&self) -> &'static str {
        "UPDATE messages SET updated_at = updated_at \
         WHERE conversation_id = $1 AND id = $2 AND deleted_at IS NULL AND kind = 'user' \
         RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Adds the reaction `emoji` of `user_id` to message `message_id`; adding it again changes nothing
/// (primary key).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddReactionQueryView {
    params: Vec<QueryParam>,
}

impl AddReactionQueryView {
    pub fn new(message_id: u64, user_id: u64, emoji: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I64(bigint_to_sql(message_id)),
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::Text(emoji.to_string()),
            ],
        }
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[0].as_i64())
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl Display for AddReactionQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddReactionQueryView: message_id={} user_id={}",
            self.message_id(),
            self.user_id()
        )
    }
}

impl ApiRequestDto for AddReactionQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO message_reactions (message_id, user_id, emoji) VALUES ($1, $2, $3) \
         ON CONFLICT DO NOTHING"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Removes the reaction `emoji` of `user_id` from message `message_id`; no row when they had not
/// reacted with it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoveReactionQueryView {
    params: Vec<QueryParam>,
}

impl RemoveReactionQueryView {
    pub fn new(message_id: u64, user_id: u64, emoji: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I64(bigint_to_sql(message_id)),
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::Text(emoji.to_string()),
            ],
        }
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[0].as_i64())
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl Display for RemoveReactionQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RemoveReactionQueryView: message_id={} user_id={}",
            self.message_id(),
            self.user_id()
        )
    }
}

impl ApiRequestDto for RemoveReactionQueryView {
    fn query_sql(&self) -> &'static str {
        "DELETE FROM message_reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3 \
         RETURNING message_id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// The reactions of message `message_id` grouped by emoji, in the order of their first reaction
/// (rows of [`crate::database::chats::get_chat::view::ReactionRow`]).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageReactionsQueryView {
    params: Vec<QueryParam>,
}

impl MessageReactionsQueryView {
    pub fn new(message_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I64(bigint_to_sql(message_id))],
        }
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[0].as_i64())
    }
}

impl Display for MessageReactionsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MessageReactionsQueryView: message_id={}",
            self.message_id()
        )
    }
}

impl ApiRequestDto for MessageReactionsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(r) FROM ( \
             SELECT mr.emoji, COUNT(*)::int AS count, \
                    jsonb_agg(mr.user_id ORDER BY mr.created_at, mr.user_id) AS user_ids, \
                    MIN(mr.created_at) AS first_at \
             FROM message_reactions mr WHERE mr.message_id = $1 GROUP BY mr.emoji \
             ORDER BY first_at, mr.emoji \
         ) r"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
