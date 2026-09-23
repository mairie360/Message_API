use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// What the caller may do on a chat: whether it exists, whether the caller is one of its (not
/// excluded) members, and whether the caller is an administrator (who bypasses membership).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ChatAccess {
    pub chat_exists: bool,
    pub is_member: bool,
    pub is_admin: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatAccessQueryView {
    params: Vec<QueryParam>,
}

impl ChatAccessQueryView {
    pub fn new(chat_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(chat_id as i32),
                QueryParam::I32(user_id as i32),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn user_id(&self) -> u64 {
        self.params[1].as_i32() as u64
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
        "SELECT jsonb_build_object( \
            'chat_exists', EXISTS(SELECT 1 FROM conversations WHERE id = $1), \
            'is_member', EXISTS( \
                SELECT 1 FROM conversation_members \
                WHERE conversation_id = $1 AND user_id = $2 AND is_excluded = FALSE), \
            'is_admin', is_admin($2))"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
