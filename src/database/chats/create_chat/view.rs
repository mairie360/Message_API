use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateChatQueryView {
    params: Vec<QueryParam>,
}

impl CreateChatQueryView {
    pub fn new(title: &str, group_id: Option<i32>) -> Self {
        Self {
            params: vec![
                QueryParam::Text(title.to_string()),
                QueryParam::OptionI32(group_id),
            ],
        }
    }

    pub fn title(&self) -> &str {
        self.params[0].as_text()
    }

    pub fn group_id(&self) -> Option<i32> {
        self.params[1].as_option_i32()
    }
}

impl Display for CreateChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CreateChatQueryView: title={} group_id={}",
            self.title(),
            self.group_id().unwrap_or_default()
        )
    }
}

impl ApiRequestDto for CreateChatQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO conversations (title, group_id) VALUES ($1, $2) RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
