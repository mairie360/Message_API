//! Test-only queries on tables the API never reads (moderation log) or writes (ACLs, granted by the
//! platform), and on `is_excluded` of a group chat, which the API only sets on direct chats. The
//! tests connect as the Postgres superuser, so the grants of `message_api` do not apply.

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

#[derive(serde::Serialize, serde::Deserialize)]
struct GrantChatDelete {
    params: Vec<QueryParam>,
}

impl Display for GrantChatDelete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GrantChatDelete: {:?}", self.params)
    }
}

impl ApiRequestDto for GrantChatDelete {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO access_control (user_id, resource_id, resource_instance_id, permission_id) \
         SELECT $2, res.id, $1, p.id \
         FROM resources res JOIN permissions p ON p.resource_id = res.id \
         WHERE res.name = 'conversations' AND p.action = 'delete'"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Grants `user_id` the `delete` right on conversation `chat_id` through an individual ACL, as
/// an administrator of the platform would.
pub async fn grant_chat_delete(db: &SmartDatabase, chat_id: u64, user_id: u64) {
    db.execute(GrantChatDelete {
        params: vec![
            QueryParam::I32(chat_id as i32),
            QueryParam::I32(user_id as i32),
        ],
    })
    .await
    .unwrap();
}

/// Any SQL the API never runs (raw fixtures, `EXPLAIN`): the statement is leaked to fit
/// `ApiRequestDto::query_sql`, which only matters in tests.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RawQuery {
    sql: String,
    params: Vec<QueryParam>,
}

impl RawQuery {
    pub fn new(sql: &str, params: Vec<QueryParam>) -> Self {
        Self {
            sql: sql.to_string(),
            params,
        }
    }

    /// `EXPLAIN (FORMAT JSON)` of `view`, with its parameters.
    pub fn explain<Q: ApiRequestDto>(view: &Q) -> Self {
        Self::new(
            &format!("EXPLAIN (FORMAT JSON) {}", view.query_sql()),
            view.query_params().to_vec(),
        )
    }
}

impl Display for RawQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RawQuery: {}", self.sql)
    }
}

impl ApiRequestDto for RawQuery {
    fn query_sql(&self) -> &'static str {
        Box::leak(self.sql.clone().into_boxed_str())
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Sets `updated_at` of the messages `ids` to `at` behind the triggers' back
/// (`session_replication_role = replica`, superuser only), to build equal timestamps.
pub async fn force_updated_at(db: &SmartDatabase, ids: &[i64], at: chrono::DateTime<chrono::Utc>) {
    let mut tx = db.begin().await.unwrap();
    tx.execute(&RawQuery::new(
        "SET LOCAL session_replication_role = replica",
        vec![],
    ))
    .await
    .unwrap();
    let list = ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
    tx.execute(&RawQuery::new(
        "UPDATE messages SET updated_at = $1 \
         WHERE id = ANY (string_to_array($2, ',')::bigint[])",
        vec![QueryParam::DateTime(at), QueryParam::Text(list)],
    ))
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

/// `unread_counters` of `user_id` in `chat_id` (0 when the row is gone).
pub async fn unread_count(db: &SmartDatabase, chat_id: u64, user_id: u64) -> i32 {
    db.fetch_scalar::<i32, _>(&RawQuery::new(
        "SELECT COALESCE((SELECT unread_count FROM unread_counters \
         WHERE conversation_id = $1 AND user_id = $2), 0)",
        vec![
            QueryParam::I32(chat_id as i32),
            QueryParam::I32(user_id as i32),
        ],
    ))
    .await
    .unwrap()
}
