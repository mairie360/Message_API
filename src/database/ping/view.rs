use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// `SELECT 1`: proves Postgres answers (readiness probe and startup check).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PingQueryView;

impl Display for PingQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PingQueryView")
    }
}

impl ApiRequestDto for PingQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT 1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &[]
    }
}
