//! Liveness and readiness probes (MAIR-423).

use std::time::Duration;

use actix_web::http::{Method, StatusCode};
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use mairie360_api_lib::test_setup::redis_setup::start_redis_container;
use message_api::endpoints::{health, ready};
use serial_test::serial;

use super::harness::request;
use crate::common::get_smart_db;

macro_rules! probe_app {
    ($state:expr) => {
        actix_web::test::init_service(
            actix_web::App::new()
                .app_data(actix_web::web::Data::new($state))
                .service(health::health)
                .service(ready::ready),
        )
        .await
    };
}

#[actix_web::test]
#[serial]
async fn ready_answers_200_when_postgres_and_redis_answer() {
    let (_container, url) = get_shared_db().await;
    let (_redis, redis) = start_redis_container().await;
    let app = probe_app!(AppState::new(redis.url.clone(), url.clone()).await);

    let (status, body) = send_json!(app, request(Method::GET, "/ready", None));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!({ "postgres": "up", "redis": "up" }));

    let (status, body) = send!(app, request(Method::GET, "/health", None));
    assert_eq!((status, body.as_str()), (StatusCode::OK, "OK"));
}

#[actix_web::test]
#[serial]
async fn ready_answers_503_naming_the_dependency_down() {
    let (_container, url) = get_shared_db().await;
    // Nothing listens on port 1.
    let app = probe_app!(AppState::new("redis://127.0.0.1:1".to_string(), url.clone()).await);

    let (status, body) = send_json!(app, request(Method::GET, "/ready", None));
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        body,
        serde_json::json!({ "postgres": "up", "redis": "down" })
    );

    // Liveness does not depend on Redis.
    let (status, _) = send!(app, request(Method::GET, "/health", None));
    assert_eq!(status, StatusCode::OK);
}

#[actix_web::test]
#[serial]
async fn startup_gives_up_on_an_unreachable_database() {
    let db = get_smart_db("postgres://user:password@127.0.0.1:1/none").await;
    let result = ready::wait_for_postgres(&db, Duration::from_secs(1)).await;
    assert!(result.is_err());

    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    assert!(ready::wait_for_postgres(&db, Duration::from_secs(5))
        .await
        .is_ok());
}
