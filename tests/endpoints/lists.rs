//! Bounded lists and per-user rate limiting (MAIR-425).

use actix_web::http::{Method, StatusCode};
use message_api::endpoints::rate_limit::{rate_limit_config, rate_limiter};
use serde_json::json;
use serial_test::serial;

use super::harness::{request, states};
use crate::common::{get_smart_db, plain_user};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[actix_web::test]
#[serial]
async fn chats_and_members_are_paginated() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
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
    assert_eq!(body["users"], json!([{ "id": user }, { "id": second }]));
    assert_eq!(body["has_more"], true);
    let uri = format!("/api/v1/{}/users/?limit=2&offset=2", chats[0]);
    let (_, body) = send_json!(app, request(Method::GET, &uri, Some(user)));
    assert_eq!(body["users"], json!([{ "id": third }]));
    assert_eq!(body["has_more"], false);

    for uri in [
        "/api/v1/?limit=0".to_string(),
        "/api/v1/?limit=101".to_string(),
        "/api/v1/?offset=-1".to_string(),
        format!("/api/v1/{}/users/?limit=101", chats[0]),
    ] {
        let (status, _) = send!(app, request(Method::GET, &uri, Some(user)));
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
    }
}

#[actix_web::test]
#[serial]
async fn rate_limit_is_per_user() {
    let (state, sse) = states().await;
    let limit = rate_limit_config(1, 2).unwrap();
    let app = actix_web::test::init_service(
        actix_web::App::new()
            .app_data(sse.clone())
            .app_data(state.clone())
            .service(
                actix_web::web::scope("/api")
                    .wrap(rate_limiter(Some(&limit)))
                    .wrap(mairie360_api_lib::security::JwtMiddleware)
                    .configure(message_api::endpoints::config),
            ),
    )
    .await;
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
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

#[test]
fn zero_per_second_disables_the_limit() {
    assert!(rate_limit_config(0, 50).is_none());
    assert!(rate_limit_config(10, 50).is_some());
}
