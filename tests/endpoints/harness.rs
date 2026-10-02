use std::sync::{Arc, Once};

use actix_web::http::Method;
use actix_web::test::TestRequest;
use actix_web::web;
use mairie360_api_lib::jwt_manager::generate_jwt;
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ADMIN_ID};
use message_api::sse::state::AppState as SseState;

/// Signing secret of the test tokens. Only read by this test binary.
const TEST_JWT_SECRET: &str = "endpoint-tests-only-jwt-secret-0123456789";

static JWT_ENV: Once = Once::new();

fn set_jwt_env() {
    JWT_ENV.call_once(|| {
        std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);
        std::env::set_var("JWT_TIMEOUT", "3600");
    });
}

/// A valid JWT for `user_id`, accepted by `JwtMiddleware`.
pub fn token(user_id: u64) -> String {
    set_jwt_env();
    generate_jwt(&user_id.to_string(), "user").expect("test JWT")
}

/// Id of the administrator seeded by the shared test database.
pub async fn admin_id() -> u64 {
    get_shared_db().await;
    *ADMIN_ID.get().expect("seeded admin") as u64
}

/// The two `web::Data` the handlers read: the lib state over the shared test database, and an SSE
/// state without Redis relay (events stay on the local bus).
pub async fn states() -> (web::Data<AppState>, web::Data<SseState>) {
    set_jwt_env();
    let (_container, url) = get_shared_db().await;
    let state = AppState::new("redis://127.0.0.1:6379".to_string(), url.clone()).await;
    let (bus, _) = tokio::sync::broadcast::channel(100);
    (
        web::Data::new(state),
        web::Data::from(Arc::new(SseState::new(bus, None))),
    )
}

/// A request on behalf of `user` (`None` = no `Authorization` header).
pub fn request(method: Method, uri: &str, user: Option<u64>) -> TestRequest {
    let request = TestRequest::default().method(method).uri(uri);
    match user {
        Some(id) => request.insert_header(("Authorization", format!("Bearer {}", token(id)))),
        None => request,
    }
}

/// Initializes the `/api` scope as mounted by `main.rs`.
macro_rules! test_app {
    ($state:expr, $sse:expr) => {
        actix_web::test::init_service(
            actix_web::App::new()
                .app_data($sse.clone())
                .app_data($state.clone())
                .service(
                    actix_web::web::scope("/api")
                        .wrap(mairie360_api_lib::security::JwtMiddleware)
                        .configure(message_api::endpoints::config),
                ),
        )
        .await
    };
}

/// Sends a `TestRequest` and returns its status and body (middleware errors included).
macro_rules! send {
    ($app:expr, $request:expr) => {{
        match actix_web::test::try_call_service(&$app, $request.to_request()).await {
            Ok(response) => {
                let status = response.status();
                let body = actix_web::test::read_body(response).await;
                (status, String::from_utf8_lossy(&body).into_owned())
            }
            Err(error) => {
                let response = error.error_response();
                let status = response.status();
                let body = actix_web::body::to_bytes(response.into_body())
                    .await
                    .unwrap_or_default();
                (status, String::from_utf8_lossy(&body).into_owned())
            }
        }
    }};
}

/// Sends a JSON request and parses the JSON body of a success.
macro_rules! send_json {
    ($app:expr, $request:expr) => {{
        let (status, body) = send!($app, $request);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        (status, json)
    }};
}
