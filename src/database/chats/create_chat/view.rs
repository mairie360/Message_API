use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct CreateChatQueryView {
    title: String,
    group_id: Option<i32>,
}

impl CreateChatQueryView {
    pub fn new(title: &str, group_id: Option<i32>) -> Self {
        Self {
            title: title.to_string(),
            group_id,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn group_id(&self) -> Option<i32> {
        self.group_id
    }
}

impl Display for CreateChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CreateChatQueryView: title={} group_id={}",
            self.title,
            self.group_id.unwrap_or_default()
        )
    }
}

impl DatabaseQueryView for CreateChatQueryView {
    fn get_request(&self) -> String {
        "INSERT INTO conversations (title, group_id) VALUES ($1, $2) RETURNING id".to_string()
    }
}
