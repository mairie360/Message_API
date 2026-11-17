//! MAIR-534: an open `GET /stream` is closed by the API when the JWT it was opened with expires (it does not wait
//! for the next revalidation), and stays open for a token that lives longer.

use std::time::{Duration, Instant};

use actix_web::http::Method;
use actix_web::test::{self, TestRequest};

use super::harness::{core_claims, now, set_jwt_env, sign_like_core, states};
use crate::common::{get_fixture_db, plain_user};

/// Returns how long the stream stayed open, whether it ended, and the `exp` of the token (epoch seconds).
async fn open_stream_with_token_expiring_in(seconds: usize) -> (Duration, bool, usize) {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let user = plain_user(&db).await;
    set_jwt_env();
    let expires_at = now() + seconds;
    let token = sign_like_core(&core_claims(user, expires_at));

    let response = test::call_service(
        &app,
        TestRequest::default()
            .method(Method::GET)
            .uri("/api/v1/stream")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    assert!(response.status().is_success(), "{}", response.status());

    // `read_body` only returns once the response stream has ended.
    let started = Instant::now();
    let ended = tokio::time::timeout(Duration::from_secs(8), test::read_body(response))
        .await
        .is_ok();
    (started.elapsed(), ended, expires_at)
}

#[actix_web::test]
async fn the_stream_ends_when_its_jwt_expires() {
    let (elapsed, ended, expires_at) = open_stream_with_token_expiring_in(2).await;
    let closed_at = now();

    assert!(
        ended,
        "the stream is still open 8 s after its token expired"
    );
    // Not before the expiry (1 s of tolerance: `exp` counts whole seconds).
    assert!(
        closed_at + 1 >= expires_at,
        "closed {} s before the token expired",
        expires_at - closed_at
    );
    assert!(
        elapsed < Duration::from_secs(6),
        "closed after {elapsed:?}, not at the expiry of the token"
    );
}

#[actix_web::test]
async fn the_stream_of_a_long_lived_token_stays_open() {
    let (elapsed, ended, _) = open_stream_with_token_expiring_in(3600).await;

    assert!(
        !ended,
        "the stream closed after {elapsed:?} for a valid token"
    );
}
