use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Hides direct chat `chat_id` for `user_id` (MAIR-478): their membership is kept, flagged
/// `is_excluded`, so the chat leaves their list but they stay its participant. The next message
/// posted in it shows it again ([`super::super::reveal_direct_chat`]). Returns no row when
/// `user_id` is not a visible member.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HideDirectChatQueryView {
    params: Vec<QueryParam>,
}

impl HideDirectChatQueryView {
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

impl Display for HideDirectChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HideDirectChatQueryView: chat_id={} user_id={}",
            self.chat_id(),
            self.user_id()
        )
    }
}

impl ApiRequestDto for HideDirectChatQueryView {
    fn query_sql(&self) -> &'static str {
        "UPDATE conversation_members SET is_excluded = TRUE \
         WHERE conversation_id = $1 AND user_id = $2 AND is_excluded = FALSE \
         RETURNING user_id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
