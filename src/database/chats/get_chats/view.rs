use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct GetChatsQueryView {
    user_id: u64,
}

impl GetChatsQueryView {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

impl Display for GetChatsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GetChatsQueryView: user_id={}", self.user_id)
    }
}

impl DatabaseQueryView for GetChatsQueryView {
    fn get_request(&self) -> String {
        "SELECT
            c.id,
            c.title,
            COALESCE(uc.unread_count, 0) as unread_count
         FROM conversations c
         INNER JOIN conversation_members cm ON c.id = cm.conversation_id
         LEFT JOIN unread_counters uc ON c.id = uc.conversation_id AND uc.user_id = cm.user_id
         WHERE cm.user_id = $1 AND cm.is_excluded = FALSE
         ORDER BY c.created_at DESC"
            .to_string()
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct GetChatsQueryResultView {
    pub id: i32,
    pub title: Option<String>,
    pub unread_count: i32,
}
