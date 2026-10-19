//! MAIR-524: incremental reads, soft delete and the events published by every message write,
//! through the real `/api` scope.

use actix_web::http::{Method, StatusCode};
use message_api::sse::state::{ChatEvent, ChatEventKind};
use serde_json::{json, Value};
use serial_test::serial;
use tokio::sync::broadcast;

use super::harness::{request, states};
use crate::common::{get_fixture_db, plain_user};

/// The cursor of a `GET /api/v1/{chat_id}/` response, as query parameters.
fn cursor_of(body: &Value) -> String {
    format!(
        "since={}&since_id={}",
        body["next_since"].as_str().unwrap().replace('+', "%2B"),
        body["next_since_id"]
    )
}

fn message_ids(body: &Value) -> Vec<u64> {
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_u64().unwrap())
        .collect()
}

/// The next event published on the bus, if any within a second.
async fn next_event(rx: &mut broadcast::Receiver<ChatEvent>) -> Option<ChatEvent> {
    tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .ok()
        .and_then(Result::ok)
}

#[actix_web::test]
#[serial]
async fn mair_524_open_chat_is_polled_from_a_cursor() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let author = plain_user(&db).await;
    let member = plain_user(&db).await;
    let mut events = sse.internal_bus.subscribe();

    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(author))
            .set_json(json!({ "name": "Polling", "members": [author, member] }))
    );
    let chat_id = body["id"].as_u64().unwrap();
    let mut posted = Vec::new();
    for content in ["Un", "Deux", "Trois"] {
        let (status, body) = send_json!(
            app,
            request(
                Method::POST,
                &format!("/api/v1/{chat_id}/messages/"),
                Some(author)
            )
            .set_json(json!({ "content": content }))
        );
        assert_eq!(status, StatusCode::OK);
        let id = body["id"].as_u64().unwrap();
        posted.push(id);
        let event = next_event(&mut events).await.expect("NEW_MSG published");
        assert_eq!(event, ChatEvent::message_created(chat_id, author, id));
    }

    // The history gives the cursor to poll from, and the new fields of every message.
    let (status, history) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(member))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(message_ids(&history), posted);
    assert_eq!(history["next_since_id"], json!(posted[2]));
    assert_eq!(history["messages"][2]["updated_at"], history["next_since"]);
    assert_eq!(history["messages"][0]["deleted"], json!(false));
    assert_eq!(history["messages"][0]["edited_at"], Value::Null);
    assert!(history["chat"]["last_activity_at"].is_string());
    let cursor = cursor_of(&history);

    // Nothing changed: the cursor is echoed.
    let (status, idle) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/?{cursor}"),
            Some(member)
        )
    );
    assert_eq!(status, StatusCode::OK);
    assert!(message_ids(&idle).is_empty());
    assert_eq!(idle["has_more"], json!(false));
    assert_eq!(idle["next_before"], Value::Null);
    assert_eq!(cursor_of(&idle), cursor);

    // An edit and a deletion, each published once committed.
    let (status, _) = send!(
        app,
        request(
            Method::PATCH,
            &format!("/api/v1/{chat_id}/messages/{}/", posted[0]),
            Some(author)
        )
        .set_json(json!({ "content": "Un (corrigé)" }))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        next_event(&mut events).await,
        Some(ChatEvent::message_edited(chat_id, author, posted[0]))
    );
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/messages/{}/", posted[1]),
            Some(author)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        next_event(&mut events).await,
        Some(ChatEvent::message_deleted(chat_id, author, posted[1]))
    );

    // One change per page: has_more until the last one.
    let (_, page) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/?{cursor}&limit=1"),
            Some(member)
        )
    );
    assert_eq!(message_ids(&page), vec![posted[0]]);
    assert_eq!(page["has_more"], json!(true));
    assert_eq!(page["messages"][0]["content"], "Un (corrigé)");
    assert!(page["messages"][0]["edited_at"].is_string());
    let (_, page) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/?{}&limit=1", cursor_of(&page)),
            Some(member)
        )
    );
    assert_eq!(message_ids(&page), vec![posted[1]]);
    assert_eq!(page["has_more"], json!(false));
    let tombstone = &page["messages"][0];
    assert_eq!(tombstone["deleted"], json!(true));
    assert_eq!(tombstone["content"], "");
    assert_eq!(tombstone["citation"], Value::Null);

    // The history no longer has the deleted message.
    let (_, history) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(member))
    );
    assert_eq!(message_ids(&history), vec![posted[0], posted[2]]);

    // A read acknowledgement is published for the reader's other streams.
    let (status, body) = send_json!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/read/"),
            Some(member)
        )
        .set_json(json!({ "readUntilMessageId": posted[2] }))
    );
    assert_eq!(status, StatusCode::OK);
    let event = next_event(&mut events).await.expect("CHAT_READ published");
    assert_eq!(event.kind, ChatEventKind::ChatRead);
    assert_eq!(
        event,
        ChatEvent::chat_read(
            chat_id,
            member,
            posted[2],
            body["unread_count"].as_i64().unwrap() as i32
        )
    );
}

#[actix_web::test]
#[serial]
async fn mair_524_tombstones_are_final() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let author = plain_user(&db).await;
    let mut events = sse.internal_bus.subscribe();

    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(author))
            .set_json(json!({ "name": "Final", "members": [author] }))
    );
    let chat_id = body["id"].as_u64().unwrap();
    let (_, body) = send_json!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/messages/"),
            Some(author)
        )
        .set_json(json!({ "content": "Bientôt supprimé" }))
    );
    let message_id = body["id"].as_u64().unwrap();
    let uri = format!("/api/v1/{chat_id}/messages/{message_id}/");
    let (status, _) = send!(app, request(Method::DELETE, &uri, Some(author)));
    assert_eq!(status, StatusCode::NO_CONTENT);
    while next_event(&mut events).await.is_some() {}

    let (status, body) = send!(
        app,
        request(Method::PATCH, &uri, Some(author)).set_json(json!({ "content": "Retour" }))
    );
    assert_eq!(
        (status, body.as_str()),
        (StatusCode::NOT_FOUND, "Unknown message.")
    );
    let (status, _) = send!(app, request(Method::DELETE, &uri, Some(author)));
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/messages/"),
            Some(author)
        )
        .set_json(json!({ "content": "Je cite", "citation": message_id }))
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
    // A refused write publishes nothing.
    assert_eq!(next_event(&mut events).await, None);
}

#[actix_web::test]
#[serial]
async fn mair_524_since_refusals() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let member = plain_user(&db).await;
    let outsider = plain_user(&db).await;
    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(member))
            .set_json(json!({ "name": "Refusals", "members": [member] }))
    );
    let chat_id = body["id"].as_u64().unwrap();
    let since = "2026-09-16T09:12:00Z";

    for (query, field) in [
        (format!("since={since}&before=118"), "since"),
        ("since_id=3".to_string(), "since_id"),
    ] {
        let (status, body) = send!(
            app,
            request(
                Method::GET,
                &format!("/api/v1/{chat_id}/?{query}"),
                Some(member)
            )
        );
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
        assert!(body.starts_with(&format!("Invalid `{field}`")), "{body}");
    }
    let (status, _) = send!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/?since=yesterday"),
            Some(member)
        )
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send!(
        app,
        request(Method::GET, "/api/v1/?since=yesterday", Some(member))
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // An outsider learns nothing, cursor or not.
    let (status, body) = send!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/?since={since}"),
            Some(outsider)
        )
    );
    assert_eq!(
        (status, body.as_str()),
        (StatusCode::NOT_FOUND, "Unknown chat.")
    );
}

#[actix_web::test]
#[serial]
async fn mair_524_chat_list_since() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let user = plain_user(&db).await;
    let mut chats = Vec::new();
    for name in ["Calme", "Animé"] {
        let (_, body) = send_json!(
            app,
            request(Method::POST, "/api/v1/", Some(user))
                .set_json(json!({ "name": name, "members": [user] }))
        );
        chats.push(body["id"].as_u64().unwrap());
    }
    let (_, list) = send_json!(app, request(Method::GET, "/api/v1/", Some(user)));
    let cursor = list["chats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|chat| chat["last_activity_at"].as_str().unwrap().to_string())
        .max()
        .unwrap();
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{}/messages/", chats[1]),
            Some(user)
        )
        .set_json(json!({ "content": "Du nouveau" }))
    );
    assert_eq!(status, StatusCode::OK);

    let (status, list) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/?since={}", cursor.replace('+', "%2B")),
            Some(user)
        )
    );
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<u64> = list["chats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|chat| chat["id"].as_u64().unwrap())
        .collect();
    assert_eq!(ids, vec![chats[1]]);
    assert_eq!(list["has_more"], json!(false));
}
