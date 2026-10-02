//! Readiness probe (MAIR-423): `/health` only says the process is up, `/ready` says it can serve.

use std::future::Future;
use std::time::Duration;

use actix_web::{get, web, HttpResponse, Responder};
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::state::AppState;
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::database::ping::view::PingQueryView;

/// Longest a dependency may take to answer before it is reported down.
pub const CHECK_TIMEOUT: Duration = Duration::from_secs(2);
/// Key read by the Redis check, under the API's key prefix (`message-api:readiness` on the
/// platform): the `message-api` ACL role may only run `GET`/`SET`/`DEL`/`EXISTS`/`EXPIRE` on its
/// own keys, `PING` included in `-@all`.
const REDIS_PROBE_KEY: &str = "readiness";

/// State of one dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DependencyStatus {
    /// Answered within 2 seconds.
    Up,
    /// Did not answer, answered an error, or took more than 2 seconds.
    Down,
}

/// Body of `GET /ready`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReadinessView {
    /// PostgreSQL: every route reads or writes it.
    #[schema(example = "up")]
    pub postgres: DependencyStatus,
    /// Redis: session revocation checks of the JWTs and the SSE relay between replicas.
    #[schema(example = "up")]
    pub redis: DependencyStatus,
}

impl ReadinessView {
    pub fn is_ready(&self) -> bool {
        self.postgres == DependencyStatus::Up && self.redis == DependencyStatus::Up
    }
}

async fn status_of<F, E>(check: F) -> DependencyStatus
where
    F: Future<Output = Result<(), E>>,
    E: std::fmt::Display,
{
    match tokio::time::timeout(CHECK_TIMEOUT, check).await {
        Ok(Ok(())) => DependencyStatus::Up,
        Ok(Err(error)) => {
            tracing::warn!(error = %error, "readiness check failed");
            DependencyStatus::Down
        }
        Err(_) => {
            tracing::warn!("readiness check timed out");
            DependencyStatus::Down
        }
    }
}

/// Whether Postgres answers `SELECT 1` within [`CHECK_TIMEOUT`].
pub async fn postgres_status(db: &SmartDatabase) -> DependencyStatus {
    status_of(async { db.fetch_scalar::<i32, _>(&PingQueryView).await.map(|_| ()) }).await
}

/// Waits until Postgres answers, at most `timeout`. Startup calls it so that an API without its
/// database exits (and is restarted) instead of serving `500`s.
///
/// # Errors
///
/// Returns a message when Postgres still does not answer after `timeout`.
pub async fn wait_for_postgres(db: &SmartDatabase, timeout: Duration) -> Result<(), String> {
    let started = tokio::time::Instant::now();
    loop {
        if postgres_status(db).await == DependencyStatus::Up {
            return Ok(());
        }
        if started.elapsed() >= timeout {
            return Err(format!(
                "PostgreSQL did not answer within {} seconds",
                timeout.as_secs()
            ));
        }
        tracing::warn!("waiting for PostgreSQL...");
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Checks both dependencies concurrently.
pub async fn readiness(state: &AppState) -> ReadinessView {
    let redis = state.get_redis();
    let (postgres, redis) = tokio::join!(
        postgres_status(state.get_smart_db()),
        status_of(async { redis.key_exist(REDIS_PROBE_KEY).await.map(|_| ()) }),
    );
    ReadinessView { postgres, redis }
}

#[utoipa::path(
    get,
    path = "ready",
    summary = "Readiness probe",
    description = "Checks that the dependencies answer: PostgreSQL (`SELECT 1`) and Redis (`EXISTS` on a key of \
                   the API's prefix), each within 2 seconds. Unauthenticated, used as the Kubernetes readiness \
                   probe and by the test stacks: a replica answering `503` is taken out of the service until it \
                   answers `200` again.\n\n\
                   `GET /health` is the liveness probe: it only says the process accepts connections and never \
                   touches the dependencies, so a database outage does not restart every replica.",
    responses(
        (
            status = 200,
            description = "PostgreSQL and Redis both answered.",
            body = ReadinessView,
            example = json!({ "postgres": "up", "redis": "up" })
        ),
        (
            status = 503,
            description = "At least one dependency is down; the body tells which (the cause is logged).",
            body = ReadinessView,
            example = json!({ "postgres": "down", "redis": "up" })
        )
    ),
    tag = "Service"
)]
#[get("/ready")]
pub async fn ready(state: web::Data<AppState>) -> impl Responder {
    let view = readiness(&state).await;
    if view.is_ready() {
        HttpResponse::Ok().json(view)
    } else {
        HttpResponse::ServiceUnavailable().json(view)
    }
}

#[derive(OpenApi)]
#[openapi(paths(ready), components(schemas(ReadinessView, DependencyStatus)))]
pub struct ReadyDoc;
