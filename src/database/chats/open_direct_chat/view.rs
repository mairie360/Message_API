use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Finds or creates **the** direct chat of `user_id` and `contact_id`, in one statement
/// (MAIR-478). `uq_conversations_direct_pair` allows a single direct chat per pair, so two
/// concurrent calls end on the same chat. `user_id` sees the chat again if they had hidden it; a
/// new chat stays hidden for `contact_id` until a message is posted in it, and an existing one is
/// left as `contact_id` has it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenDirectChatQueryView {
    params: Vec<QueryParam>,
}

impl OpenDirectChatQueryView {
    pub fn new(user_id: u64, contact_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(user_id)),
                QueryParam::I32(id_to_sql(contact_id)),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn contact_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl Display for OpenDirectChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OpenDirectChatQueryView: user_id={} contact_id={}",
            self.user_id(),
            self.contact_id()
        )
    }
}

impl ApiRequestDto for OpenDirectChatQueryView {
    fn query_sql(&self) -> &'static str {
        // The no-op DO UPDATE makes RETURNING give the existing chat too; `xmax = 0` only holds for
        // a row this statement inserted. An unknown contact fails the foreign key.
        "WITH chat AS ( \
             INSERT INTO conversations (title, kind, created_by, direct_user_low, direct_user_high) \
             VALUES (NULL, 'direct', $1, LEAST($1::int, $2::int), GREATEST($1::int, $2::int)) \
             ON CONFLICT (direct_user_low, direct_user_high) \
             DO UPDATE SET direct_user_low = EXCLUDED.direct_user_low \
             RETURNING id, (xmax = 0) AS created \
         ), members AS ( \
             INSERT INTO conversation_members (conversation_id, user_id, is_excluded) \
             SELECT chat.id, m.user_id, m.hidden \
             FROM chat, (VALUES ($1::int, FALSE), ($2::int, TRUE)) AS m(user_id, hidden) \
             ON CONFLICT (conversation_id, user_id) \
             DO UPDATE SET is_excluded = conversation_members.is_excluded AND EXCLUDED.is_excluded \
         ) \
         SELECT to_jsonb(chat) FROM chat"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenDirectChatQueryResultView {
    pub id: i32,
    /// `true` when the statement created the chat, `false` when it already existed.
    pub created: bool,
}
