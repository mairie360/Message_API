use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// What the caller may do on a chat: whether it exists, whether the caller is one of its (not
/// excluded) members, whether the caller created it (`conversations.created_by`) and whether the
/// caller is an administrator (who may read and moderate any chat).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ChatAccess {
    pub chat_exists: bool,
    pub is_member: bool,
    pub is_creator: bool,
    pub is_admin: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatAccessQueryView {
    params: Vec<QueryParam>,
    /// Lock the chat and the caller's membership until the end of the transaction.
    lock: bool,
}

impl ChatAccessQueryView {
    pub fn new(chat_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::I32(id_to_sql(user_id)),
            ],
            lock: false,
        }
    }

    /// Same answer as [`Self::new`], for a write run in the same transaction: the chat row and the
    /// caller's membership row are locked (`FOR KEY SHARE`) until it ends, so the chat cannot be
    /// deleted nor the caller removed between the check and the write. Non-key updates (exclusion
    /// flag, title) are not blocked.
    pub fn locking(chat_id: u64, user_id: u64) -> Self {
        Self {
            lock: true,
            ..Self::new(chat_id, user_id)
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn user_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl Display for ChatAccessQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChatAccessQueryView: chat_id={} user_id={}",
            self.chat_id(),
            self.user_id()
        )
    }
}

impl ApiRequestDto for ChatAccessQueryView {
    fn query_sql(&self) -> &'static str {
        if self.lock {
            // Locking clauses are not allowed on the nullable side of an outer join: each row is
            // locked in its own sub-select, which yields no row when it does not exist.
            return "SELECT jsonb_build_object( \
                'chat_exists', c.id IS NOT NULL, \
                'is_member', m.user_id IS NOT NULL, \
                'is_creator', COALESCE(c.created_by = $2, FALSE), \
                'is_admin', is_admin($2)) \
             FROM (SELECT 1) AS one \
             LEFT JOIN LATERAL ( \
                SELECT id, created_by FROM conversations WHERE id = $1 FOR KEY SHARE) c ON TRUE \
             LEFT JOIN LATERAL ( \
                SELECT user_id FROM conversation_members \
                WHERE conversation_id = $1 AND user_id = $2 AND is_excluded = FALSE \
                FOR KEY SHARE) m ON TRUE";
        }
        "SELECT jsonb_build_object( \
            'chat_exists', EXISTS(SELECT 1 FROM conversations WHERE id = $1), \
            'is_member', EXISTS( \
                SELECT 1 FROM conversation_members \
                WHERE conversation_id = $1 AND user_id = $2 AND is_excluded = FALSE), \
            'is_creator', EXISTS(SELECT 1 FROM conversations WHERE id = $1 AND created_by = $2), \
            'is_admin', is_admin($2))"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
