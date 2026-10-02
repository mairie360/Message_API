//! Test-only queries on tables the API never reads (moderation log) or flags it never writes
//! (`is_excluded`). The tests connect as the Postgres superuser, so the grants of `message_api`
//! do not apply.

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::smart_db::SmartDatabase;
use std::fmt::Display;

#[derive(serde::Serialize, serde::Deserialize)]
struct ModerationLogCount {
    params: Vec<QueryParam>,
}

impl Display for ModerationLogCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ModerationLogCount: {:?}", self.params)
    }
}

impl ApiRequestDto for ModerationLogCount {
    fn query_sql(&self) -> &'static str {
        "SELECT count(*)::int FROM messaging_moderation_log \
         WHERE conversation_id = $1 AND action = $2 AND performed_by = $3"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Rows of `messaging_moderation_log` for this chat, action and performer.
pub async fn moderation_log_count(
    db: &SmartDatabase,
    chat_id: u64,
    action: &str,
    performed_by: u64,
) -> i32 {
    db.fetch_scalar::<i32, _>(&ModerationLogCount {
        params: vec![
            QueryParam::I32(chat_id as i32),
            QueryParam::Text(action.to_string()),
            QueryParam::I32(performed_by as i32),
        ],
    })
    .await
    .unwrap()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ExcludeMember {
    params: Vec<QueryParam>,
}

impl Display for ExcludeMember {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExcludeMember: {:?}", self.params)
    }
}

impl ApiRequestDto for ExcludeMember {
    fn query_sql(&self) -> &'static str {
        "UPDATE conversation_members SET is_excluded = TRUE \
         WHERE conversation_id = $1 AND user_id = $2"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Flags `user_id` as excluded from chat `chat_id`.
pub async fn exclude_member(db: &SmartDatabase, chat_id: u64, user_id: u64) {
    db.execute(ExcludeMember {
        params: vec![
            QueryParam::I32(chat_id as i32),
            QueryParam::I32(user_id as i32),
        ],
    })
    .await
    .unwrap();
}
