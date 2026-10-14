//! What each route lets its legitimate callers do.

use actix_web::http::{Method, StatusCode};
use serde_json::json;
use serial_test::serial;

use super::harness::{admin_id, request, states};
use crate::common::{get_smart_db, grant_chat_delete, plain_user};
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

/// MAIR-426: `<` and `>` are ordinary characters, stored and served back as sent (JSON with
/// `nosniff`, escaping is the fronts' job).
#[actix_web::test]
#[serial]
async fn angle_brackets_are_accepted_in_free_text() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let user = plain_user(&db).await;

    let name = "Budget > 10 000 € -> <validé>";
    let (status, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(user))
            .set_json(json!({ "name": name, "members": [] }))
    );
    assert_eq!(status, StatusCode::OK);
    let chat_id = body["id"].as_u64().unwrap();

    let content = "a < b && b > c <3 <script>alert(1)</script>";
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/messages/"),
            Some(user)
        )
        .set_json(json!({ "content": content }))
    );
    assert_eq!(status, StatusCode::OK);

    let (_, body) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(user))
    );
    assert_eq!(body["messages"][0]["content"], content);
    let (_, body) = send_json!(app, request(Method::GET, "/api/v1/", Some(user)));
    assert!(body["chats"]
        .as_array()
        .unwrap()
        .iter()
        .any(|chat| chat["id"] == chat_id && chat["name"] == name));
}

/// The chat `chat_id` as `GET /api/v1/` lists it for `user` (`Null` when they do not see it).
macro_rules! listed {
    ($app:expr, $user:expr, $chat_id:expr) => {{
        let (_, body) = send_json!($app, request(Method::GET, "/api/v1/", Some($user)));
        body["chats"]
            .as_array()
            .unwrap()
            .iter()
            .find(|chat| chat["id"] == $chat_id)
            .cloned()
            .unwrap_or_default()
    }};
}

/// MAIR-478: one direct chat per pair; hiding it keeps the contact, a message shows it again.
#[actix_web::test]
#[serial]
async fn direct_chat_is_reused_and_shown_again_by_a_message() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (alice, bob) = (plain_user(&db).await, plain_user(&db).await);

    let (status, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": bob }))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["created"], true);
    let chat_id = body["id"].as_u64().unwrap();

    let chat = listed!(app, alice, chat_id);
    assert_eq!(chat["kind"], "direct");
    assert_eq!(chat["contact_id"], bob);
    // The name of a direct chat is the other participant's (the test users are all "Chat Member").
    assert_eq!(chat["name"], "Chat Member");
    assert_eq!(chat["member_count"], 2);
    // Bob only sees it with the first message.
    assert!(listed!(app, bob, chat_id).is_null());
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/messages/"),
            Some(alice)
        )
        .set_json(json!({ "content": "Bonjour Bob" }))
    );
    assert_eq!(status, StatusCode::OK);
    let chat = listed!(app, bob, chat_id);
    assert_eq!(
        (chat["contact_id"].as_u64(), chat["unread_count"].as_i64()),
        (Some(alice), Some(1))
    );

    // Bob hides it: Alice keeps seeing Bob as her contact.
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/users/{bob}/"),
            Some(bob)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(listed!(app, bob, chat_id).is_null());
    assert_eq!(listed!(app, alice, chat_id)["contact_id"], bob);

    // Writing to Bob again reuses the chat and shows it to him, history included.
    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": bob }))
    );
    assert_eq!(
        (body["id"].as_u64(), body["created"].as_bool()),
        (Some(chat_id), Some(false))
    );
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/messages/"),
            Some(alice)
        )
        .set_json(json!({ "content": "Tu es là ?" }))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed!(app, bob, chat_id)["contact_id"], alice);
    let (_, body) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(bob))
    );
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);

    // Bob opening it from his side lands on the same chat.
    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(bob))
            .set_json(json!({ "contact_id": alice }))
    );
    assert_eq!(body["id"].as_u64(), Some(chat_id));

    // Both hide it: it is deleted, and the pair starts over.
    for user in [alice, bob] {
        let (status, _) = send!(
            app,
            request(
                Method::DELETE,
                &format!("/api/v1/{chat_id}/users/{user}/"),
                Some(user)
            )
        );
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    let (status, _) = send!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/"),
            Some(admin_id().await)
        )
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": bob }))
    );
    assert_eq!(body["created"], true);
}

/// A group chat is deleted by its creator, an administrator or an agent granted the right.
#[actix_web::test]
#[serial]
async fn group_chat_is_deleted_by_its_creator_or_an_agent_granted_the_right() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (creator, member, granted) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );

    let create = |name: &str| {
        request(Method::POST, "/api/v1/", Some(creator))
            .set_json(json!({ "name": name, "members": [member] }))
    };
    let (_, body) = send_json!(app, create("Service urbanisme"));
    let chat_id = body["id"].as_u64().unwrap();
    assert_eq!(listed!(app, member, chat_id)["kind"], "group");
    assert!(listed!(app, member, chat_id)["contact_id"].is_null());

    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/"),
            Some(creator)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        crate::common::moderation_log_count(&db, chat_id, "DELETE_CONVERSATION", creator).await,
        1
    );
    assert!(listed!(app, member, chat_id).is_null());

    // An agent granted `delete` on the conversation deletes it without being a member.
    let (_, body) = send_json!(app, create("Astreinte week-end"));
    let chat_id = body["id"].as_u64().unwrap();
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/"),
            Some(granted)
        )
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
    grant_chat_delete(&db, chat_id, granted).await;
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/"),
            Some(granted)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
}
