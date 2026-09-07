use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatchMessageQueryView {
    params: Vec<QueryParam>,
}

impl PatchMessageQueryView {
    pub fn new(message_id: u64, content: &str) -> Self {
        Self {
            params: vec![
                QueryParam::I32(message_id as i32),
                QueryParam::Text(content.to_string()),
            ],
        }
    }

    pub fn message_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn content(&self) -> &str {
        self.params[1].as_text()
    }
}

impl Display for PatchMessageQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PatchMessageQueryView: message_id={}, content={}",
            self.message_id(),
            self.content()
        )
    }
}

impl ApiRequestDto for PatchMessageQueryView {
    fn query_sql(&self) -> &'static str {
        "UPDATE messages SET content = $2 WHERE id = $1 RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
