//! What each route lets its legitimate callers do.

use actix_web::http::{Method, StatusCode};
use serde_json::json;
use serial_test::serial;

use super::harness::{admin_id, request, states};
use crate::common::{get_smart_db, plain_user};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[actix_web::test]
#[serial]
async fn chat_lifecycle_through_every_route() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let creator = plain_user(&db).await;
    let member = plain_user(&db).await;
    let newcomer = plain_user(&db).await;
    let admin = admin_id().await;

    let (status, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(creator))
            .set_json(json!({ "name": "Service urbanisme", "members": [member, creator] }))
    );
    assert_eq!(status, StatusCode::OK);
    let chat_id = body["id"].as_u64().unwrap();

    let (status, body) = send_json!(app, request(Method::GET, "/api/v1/", Some(member)));
    assert_eq!(status, StatusCode::OK);
    assert!(body["chats"]
        .as_array()
        .unwrap()
        .iter()
        .any(|chat| chat["id"] == chat_id));

    let (status, body) = send_json!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/messages/"),
            Some(creator)
        )
        .set_json(json!({ "content": "La réunion est décalée à 15h." }))
    );
    assert_eq!(status, StatusCode::OK);
    let first = body["id"].as_u64().unwrap();

    let (status, body) = send_json!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/messages/"),
            Some(member)
        )
        .set_json(json!({ "content": "Bien noté.", "citation": first }))
    );
    assert_eq!(status, StatusCode::OK);
    let reply = body["id"].as_u64().unwrap();

    let (status, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/?limit=1"),
            Some(member)
        )
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["has_more"], true);
    assert_eq!(body["messages"][0]["id"], reply);
    assert_eq!(body["messages"][0]["citation"], first);

    let (status, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/users/"),
            Some(member)
        )
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["users"].as_array().unwrap().len(), 2);

    let (status, _) = send!(
        app,
        request(
            Method::PATCH,
            &format!("/api/v1/{chat_id}/messages/{first}/"),
            Some(creator)
        )
        .set_json(json!({ "content": "La réunion est décalée à 16h." }))
    );
    assert_eq!(status, StatusCode::OK);

    let (status, body) = send_json!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/read/"),
            Some(creator)
        )
        .set_json(json!({ "readUntilMessageId": reply }))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["unread_count"], 0);

    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/users/"),
            Some(creator)
        )
        .set_json(json!({ "users_id": [newcomer] }))
    );
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/messages/{first}/"),
            Some(creator)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/users/{newcomer}/"),
            Some(creator)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Administrators read and moderate any chat.
    let (status, _) = send!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(admin))
    );
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/messages/{reply}/"),
            Some(admin)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send!(
        app,
        request(Method::DELETE, &format!("/api/v1/{chat_id}/"), Some(admin))
    );
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = send!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(creator))
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[actix_web::test]
#[serial]
async fn last_member_leaving_deletes_the_chat() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let alone = plain_user(&db).await;
    let admin = admin_id().await;

    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(alone))
            .set_json(json!({ "name": "", "members": [] }))
    );
    let chat_id = body["id"].as_u64().unwrap();

    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/users/{alone}/"),
            Some(alone)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = send!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(admin))
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[actix_web::test]
#[serial]
async fn stream_opens_an_event_stream() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let user = plain_user(&db).await;

    // The body never ends: only the head of the response is read.
    let response = actix_web::test::call_service(
        &app,
        request(Method::GET, "/api/v1/stream", Some(user)).to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );
    assert_eq!(sse.online_agents.get(&user).map(|c| c.len()), Some(1));
}
