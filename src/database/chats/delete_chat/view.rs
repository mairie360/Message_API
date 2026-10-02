use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Deletes chat `chat_id` (members and messages cascade) on behalf of the administrator
/// `performed_by`, and appends a `DELETE_CONVERSATION` row (with the title) to
/// `messaging_moderation_log` in the same statement.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteChatQueryView {
    params: Vec<QueryParam>,
}

impl DeleteChatQueryView {
    pub fn new(chat_id: u64, performed_by: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(chat_id as i32),
                QueryParam::I32(performed_by as i32),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn performed_by(&self) -> u64 {
        self.params[1].as_i32() as u64
    }
}

impl Display for DeleteChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DeleteChatQueryView: chat_id={} performed_by={}",
            self.chat_id(),
            self.performed_by()
        )
    }
}

impl ApiRequestDto for DeleteChatQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH deleted AS ( \
             DELETE FROM conversations WHERE id = $1 RETURNING id, title \
         ), logged AS ( \
             INSERT INTO messaging_moderation_log (action, conversation_id, content, performed_by) \
             SELECT 'DELETE_CONVERSATION', id, title, $2 FROM deleted \
         ) \
         SELECT id FROM deleted"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
