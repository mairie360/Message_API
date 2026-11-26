//! MAIR-1219: a reply keeps its citation once the quoted message is deleted, and posting a reply
//! returns the message with its quote, through the real `/api` scope.

use actix_web::http::{Method, StatusCode};
use serde_json::{json, Value};

use super::harness::{request, states};
use crate::common::{get_fixture_db, plain_user};

fn message<'a>(body: &'a Value, id: &Value) -> &'a Value {
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| &m["id"] == id)
        .expect("message in the page")
}

#[actix_web::test]
async fn mair_1219_reply_keeps_its_citation_once_the_quoted_message_is_deleted() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let author = plain_user(&db).await;
    let member = plain_user(&db).await;
    let (_, chat) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(author))
            .set_json(json!({ "name": "Deleted quotes", "members": [author, member] }))
    );
    let chat_id = chat["id"].as_u64().unwrap();
    let messages_uri = format!("/api/v1/{chat_id}/messages/");

    // A, then B answering A: the POST answer carries B with its quote at once.
    let (status, posted_a) = send_json!(
        app,
        request(Method::POST, &messages_uri, Some(author))
            .set_json(json!({ "content": "Le permis de la rue Pasteur ?" }))
    );
    assert_eq!(status, StatusCode::OK);
    let a = posted_a["id"].clone();
    assert_eq!(posted_a["message"]["id"], a);
    assert_eq!(posted_a["message"]["quoted"], Value::Null);
    let (status, posted_b) = send_json!(
        app,
        request(Method::POST, &messages_uri, Some(member))
            .set_json(json!({ "content": "Accordé hier.", "citation": a }))
    );
    assert_eq!(status, StatusCode::OK);
    let b = posted_b["id"].clone();
    assert_eq!(posted_b["message"]["id"], b);
    assert_eq!(posted_b["message"]["content"], "Accordé hier.");
    assert_eq!(posted_b["message"]["citation"], a);
    assert_eq!(
        posted_b["message"]["quoted"],
        json!({ "id": a, "sender_id": author, "excerpt": "Le permis de la rue Pasteur ?", "deleted": false })
    );

    // A is deleted.
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/messages/{a}/"),
            Some(author)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);

    // After a reload, B still says it answered a message, now deleted, without its content.
    let (status, history) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(member))
    );
    assert_eq!(status, StatusCode::OK);
    let reply = message(&history, &b);
    assert_eq!(reply["citation"], a);
    assert_eq!(reply["quoted"], json!({ "id": a, "deleted": true }));
    assert!(
        !history.to_string().contains("Le permis de la rue Pasteur"),
        "the deleted content never leaks"
    );

    // A deleted message still cannot be quoted.
    let (status, _) = send!(
        app,
        request(Method::POST, &messages_uri, Some(member))
            .set_json(json!({ "content": "Je cite", "citation": a }))
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
