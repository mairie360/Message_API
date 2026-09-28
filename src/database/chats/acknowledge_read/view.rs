use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Acknowledges the messages of chat `chat_id` up to `message_id` (included) for `user_id` and
/// returns the number of messages still unread by that agent.
///
/// The work is done by `fn_acknowledge_read`: the read cursor only moves forward, so a repeated or
/// stale acknowledgement is a no-op that returns the current count, and messages sent after the
/// cursor are never cleared. No row (`DbError::NotFound`) when `message_id` does not belong to the
/// chat.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcknowledgeReadQueryView {
    params: Vec<QueryParam>,
}

impl AcknowledgeReadQueryView {
    pub fn new(chat_id: u64, user_id: u64, message_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(user_id as i32),
                QueryParam::I32(chat_id as i32),
                QueryParam::I64(message_id as i64),
            ],
        }
    }

    pub fn user_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn chat_id(&self) -> u64 {
        self.params[1].as_i32() as u64
    }

    pub fn message_id(&self) -> u64 {
        self.params[2].as_i64() as u64
    }
}

impl Display for AcknowledgeReadQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AcknowledgeReadQueryView: chat_id={} user_id={} message_id={}",
            self.chat_id(),
            self.user_id(),
            self.message_id()
        )
    }
}

impl ApiRequestDto for AcknowledgeReadQueryView {
    fn query_sql(&self) -> &'static str {
        // The function answers NULL for a message of another chat: filtering it out turns that
        // into "no row", which the handler maps to a 404 like the other unknown-message cases.
        "SELECT unread FROM (SELECT fn_acknowledge_read($1, $2, $3) AS unread) ack \
         WHERE unread IS NOT NULL"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
