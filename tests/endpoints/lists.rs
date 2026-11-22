//! Bounded lists and per-user rate limiting.

use actix_web::http::{Method, StatusCode};
use mairie360_api_lib::security::{
    rate_limit_middleware, JwtMiddleware, RateLimitConfig, RateLimiter,
};
use serde_json::json;
use std::num::NonZeroU32;

/// The lib's defaults (`RATE_LIMIT_PER_SECOND` / `RATE_LIMIT_BURST` unset).
const DEFAULT_PER_SECOND: u64 = 20;
const DEFAULT_BURST: u32 = 50;

use super::harness::{request, states};
use crate::common::{get_fixture_db, plain_user};

#[actix_web::test]
async fn chats_and_members_are_paginated() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let user = plain_user(&db).await;
    let (second, third) = (plain_user(&db).await, plain_user(&db).await);

    let mut chats = Vec::new();
    for name in ["Premier", "Deuxième", "Troisième"] {
        let (_, body) = send_json!(
            app,
            request(Method::POST, "/api/v1/", Some(user))
                .set_json(json!({ "name": name, "members": [second, third] }))
        );
        chats.push(body["id"].as_u64().unwrap());
    }

    // Newest first, two per page.
    let (status, body) = send_json!(app, request(Method::GET, "/api/v1/?limit=2", Some(user)));
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<u64> = body["chats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|chat| chat["id"].as_u64().unwrap())
        .collect();
    assert_eq!(ids, vec![chats[2], chats[1]]);
    assert_eq!(body["has_more"], true);
    let (_, body) = send_json!(
        app,
        request(Method::GET, "/api/v1/?limit=2&offset=2", Some(user))
    );
    assert_eq!(body["chats"][0]["id"], chats[0]);
    assert_eq!(body["has_more"], false);

    // Members by increasing id.
    let uri = format!("/api/v1/{}/users/?limit=2", chats[0]);
    let (_, body) = send_json!(app, request(Method::GET, &uri, Some(user)));
    assert_eq!(
        body["users"],
        json!([
            { "id": user, "first_name": "Chat", "last_name": "Member", "is_admin": true },
            { "id": second, "first_name": "Chat", "last_name": "Member", "is_admin": false }
        ])
    );
    assert_eq!(body["has_more"], true);
    let uri = format!("/api/v1/{}/users/?limit=2&offset=2", chats[0]);
    let (_, body) = send_json!(app, request(Method::GET, &uri, Some(user)));
    assert_eq!(
        body["users"],
        json!([{ "id": third, "first_name": "Chat", "last_name": "Member", "is_admin": false }])
    );
    assert_eq!(body["has_more"], false);

    for uri in [
        "/api/v1/?limit=0".to_string(),
        "/api/v1/?limit=101".to_string(),
        "/api/v1/?offset=-1".to_string(),
        format!("/api/v1/?search={}", "a".repeat(151)),
        "/api/v1/?search=a%00b".to_string(),
        format!("/api/v1/{}/users/?limit=101", chats[0]),
    ] {
        let (status, _) = send!(app, request(Method::GET, &uri, Some(user)));
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
    }
}

#[actix_web::test]
async fn rate_limit_is_per_user() {
    let (state, sse) = states().await;
    let limiter = RateLimiter::new(RateLimitConfig {
        per_second: NonZeroU32::MIN,
        burst: NonZeroU32::new(2).unwrap(),
        trust_forwarded: false,
    });
    let app = actix_web::test::init_service(
        actix_web::App::new()
            .app_data(sse.clone())
            .app_data(state.clone())
            .app_data(limiter)
            .service(
                actix_web::web::scope("/api")
                    .wrap(actix_web::middleware::from_fn(rate_limit_middleware))
                    .wrap(JwtMiddleware)
                    .configure(message_api::endpoints::config),
            ),
    )
    .await;
    let db = get_fixture_db().await;
    let (busy, other) = (plain_user(&db).await, plain_user(&db).await);

    for _ in 0..2 {
        let (status, _) = send!(app, request(Method::GET, "/api/v1/", Some(busy)));
        assert_eq!(status, StatusCode::OK);
    }
    let (status, body) = send!(app, request(Method::GET, "/api/v1/", Some(busy)));
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(body.contains("retry"), "{body}");

    // Another user (behind the same BFF address) is not affected.
    let (status, _) = send!(app, request(Method::GET, "/api/v1/", Some(other)));
    assert_eq!(status, StatusCode::OK);
}

/// The production budget: `RateLimiter::from_env()` without overrides, mounted like `main.rs`. A user gets the whole burst, then `429` with a numeric `Retry-After` of at least one
/// second, while another user is still served.
#[actix_web::test]
async fn the_production_budget_refuses_a_user_past_its_burst() {
    std::env::remove_var("RATE_LIMIT_PER_SECOND");
    std::env::remove_var("RATE_LIMIT_BURST");
    std::env::remove_var("RATE_LIMIT_TRUST_FORWARDED");
    let (state, sse) = states().await;
    let app = actix_web::test::init_service(
        actix_web::App::new()
            .app_data(sse.clone())
            .app_data(state.clone())
            .app_data(RateLimiter::from_env())
            .service(
                actix_web::web::scope("/api")
                    .wrap(actix_web::middleware::from_fn(rate_limit_middleware))
                    .wrap(JwtMiddleware)
                    .configure(message_api::endpoints::config),
            ),
    )
    .await;
    let db = get_fixture_db().await;
    let (busy, other) = (plain_user(&db).await, plain_user(&db).await);

    let started = std::time::Instant::now();
    let mut served: u64 = 0;
    let refused = loop {
        let response = actix_web::test::call_service(
            &app,
            request(Method::GET, "/api/v1/", Some(busy)).to_request(),
        )
        .await;
        if response.status() != StatusCode::OK {
            break response;
        }
        served += 1;
        assert!(served < 10_000, "the limiter never refused");
    };
    // The bucket refills while the burst is sent.
    let refilled =
        u64::try_from(started.elapsed().as_millis()).unwrap() * DEFAULT_PER_SECOND / 1000 + 1;
    let burst = u64::from(DEFAULT_BURST);
    assert!(
        served >= burst && served <= burst + refilled,
        "{served} requests served before the 429"
    );
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after: u64 = refused
        .headers()
        .get("retry-after")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(retry_after >= 1, "Retry-After: {retry_after}");
    assert_eq!(
        refused.headers().get("x-ratelimit-after"),
        refused.headers().get("retry-after")
    );

    let (status, _) = send!(app, request(Method::GET, "/api/v1/", Some(other)));
    assert_eq!(status, StatusCode::OK);
}

/// A disabled limiter (`RATE_LIMIT_PER_SECOND=0`, set by the ZAP and k6 stacks) lets every request
/// through.
#[actix_web::test]
async fn mair_688_a_disabled_limiter_lets_every_request_through() {
    let (state, sse) = states().await;
    let app = actix_web::test::init_service(
        actix_web::App::new()
            .app_data(sse.clone())
            .app_data(state.clone())
            .app_data(RateLimiter::disabled())
            .service(
                actix_web::web::scope("/api")
                    .wrap(actix_web::middleware::from_fn(rate_limit_middleware))
                    .wrap(JwtMiddleware)
                    .configure(message_api::endpoints::config),
            ),
    )
    .await;
    let db = get_fixture_db().await;
    let user = plain_user(&db).await;
    for _ in 0..10 {
        let (status, _) = send!(app, request(Method::GET, "/api/v1/", Some(user)));
        assert_eq!(status, StatusCode::OK);
    }
}
