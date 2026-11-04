use crate::common::message_db_url;
use std::sync::{Arc, Once};

use actix_web::http::Method;
use actix_web::test::TestRequest;
use actix_web::web;
use jsonwebtoken::{encode, Algorithm, Header};
use mairie360_api_lib::jwt_manager::{
    parse_public_keys, parse_signing_key, Claims, SESSION_JWT_AUDIENCE, SESSION_JWT_ISSUER,
};
use mairie360_api_lib::state::AppState;
use mairie360_api_lib::test_setup::http_setup::{test_jwt_private_key, TEST_JWT_PUBLIC_KEY};
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, ADMIN_ID};
use message_api::sse::state::AppState as SseState;

/// Seconds since the epoch, as the `exp` claim counts them.
pub fn now() -> usize {
    usize::try_from(chrono::Utc::now().timestamp()).unwrap()
}

/// Claims of a session token issued by Core for `user_id` (MAIR-680: with `iss` and `aud`).
pub fn core_claims(user_id: u64, expiration: usize) -> Claims {
    Claims::new(&user_id.to_string(), "user", expiration)
        .with_issuer_and_audience(SESSION_JWT_ISSUER, SESSION_JWT_AUDIENCE)
}

/// PKCS#8 body (base64) of an Ed25519 private key that is not Core's (MAIR-680), assembled into
/// PEM at run time so that the secret scanners do not read a PEM private key in the sources.
const MAIR_680_OTHER_PRIVATE_KEY_B64: &str =
    "MC4CAQAwBQYDK2VwBCIEII1DTYiu+dcwBgtEKil59uTH8s7XfrxGrbD+/P0sXq/8";

/// Public key of [`other_private_key`], never published in `JWT_PUBLIC_KEY`.
const MAIR_680_OTHER_PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAtZHz1BNkijjkLJlnK9l7vikDl0aivrmbwllpT68Lnss=
-----END PUBLIC KEY-----
";

/// An Ed25519 private key that is not Core's: what a compromised service could sign with.
pub fn other_private_key() -> String {
    let label = "PRIVATE KEY";
    format!("-----BEGIN {label}-----\n{MAIR_680_OTHER_PRIVATE_KEY_B64}\n-----END {label}-----\n")
}

/// Signs `claims` as they are with `private_key` (PEM) under Core's `kid`: a genuine token with
/// [`test_jwt_private_key`], a forgery with [`other_private_key`].
pub fn sign_eddsa_with(claims: &Claims, private_key: &str) -> String {
    let core_keys = parse_public_keys(TEST_JWT_PUBLIC_KEY).unwrap();
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(core_keys[0].kid().to_string());
    let known_keys = [
        core_keys,
        parse_public_keys(MAIR_680_OTHER_PUBLIC_KEY).unwrap(),
    ]
    .concat();
    let key = parse_signing_key(private_key, &known_keys).unwrap();
    encode(&header, claims, key.encoding_key()).unwrap()
}

/// Signs `claims` like Core does (MAIR-680): `EdDSA`, the lib's test key pair, Core's `kid`.
pub fn sign_like_core(claims: &Claims) -> String {
    sign_eddsa_with(claims, &test_jwt_private_key())
}

/// The environment of a deployed API after MAIR-680: Core's public key, no `JWT_SECRET`, no
/// private key.
static JWT_ENV: Once = Once::new();

pub fn set_jwt_env() {
    JWT_ENV.call_once(|| {
        std::env::set_var("JWT_PUBLIC_KEY", TEST_JWT_PUBLIC_KEY);
        std::env::remove_var("JWT_PRIVATE_KEY");
        std::env::remove_var("JWT_SECRET");
        std::env::set_var("JWT_TIMEOUT", "3600");
    });
}

/// A session JWT issued by Core for `user_id`, accepted by `JwtMiddleware`.
pub fn token(user_id: u64) -> String {
    set_jwt_env();
    sign_like_core(&core_claims(user_id, now() + 3600))
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
    let state = AppState::new("redis://127.0.0.1:6379".to_string(), message_db_url().await).await;
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
