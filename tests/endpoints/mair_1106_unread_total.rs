//! MAIR-1106: `GET /api/v1/unread/` answers the caller's unread total over the chats of their list,
//! for the badges of the navigation and of the dashboard.

use actix_web::http::{Method, StatusCode};
use serde_json::{json, Value};

use super::harness::{request, states};
use crate::common::{get_fixture_db, named_user};

fn counts(body: &Value) -> (u64, u64) {
    (
        body["unread_count"].as_u64().unwrap(),
        body["unread_chats"].as_u64().unwrap(),
    )
}

#[actix_web::test]
async fn unread_total_sums_the_counters_of_the_callers_visible_chats() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    // Fresh users: their totals only hold what this test posts.
    let alice = named_user(&db, "Alice", "Unread").await;
    let bob = named_user(&db, "Bob", "Unread").await;
    let carol = named_user(&db, "Carole", "Unread").await;

    let unread = |user: u64| request(Method::GET, "/api/v1/unread/", Some(user));

    let (status, body) = send_json!(app, unread(bob));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(counts(&body), (0, 0), "no chat yet: {body}");

    let (_, group) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(alice))
            .set_json(json!({ "name": "Badge non lus", "members": [bob, carol] }))
    );
    let group_id = group["id"].as_u64().unwrap();
    let post = |chat: u64, content: &str| {
        request(
            Method::POST,
            &format!("/api/v1/{chat}/messages/"),
            Some(alice),
        )
        .set_json(json!({ "content": content }))
    };
    send_json!(app, post(group_id, "Premier"));
    let (_, last) = send_json!(app, post(group_id, "Second"));
    let last_group_message = last["id"].as_u64().unwrap();

    let (_, direct) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": bob }))
    );
    let direct_id = direct["id"].as_u64().unwrap();
    send_json!(app, post(direct_id, "En direct"));

    let (_, body) = send_json!(app, unread(bob));
    assert_eq!(
        counts(&body),
        (3, 2),
        "two in the group, one in the direct chat: {body}"
    );
    let (_, body) = send_json!(app, unread(carol));
    assert_eq!(counts(&body), (2, 1), "{body}");
    let (_, body) = send_json!(app, unread(alice));
    assert_eq!(
        counts(&body),
        (0, 0),
        "the author's own messages are not unread: {body}"
    );

    // Reading never changes on a GET; acknowledging the group lowers the total.
    let (_, body) = send_json!(app, unread(bob));
    assert_eq!(counts(&body), (3, 2), "{body}");
    let (status, _) = send_json!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{group_id}/read/"),
            Some(bob)
        )
        .set_json(json!({ "readUntilMessageId": last_group_message }))
    );
    assert_eq!(status, StatusCode::OK);
    let (_, body) = send_json!(app, unread(bob));
    assert_eq!(
        counts(&body),
        (1, 1),
        "only the direct chat is left: {body}"
    );

    // A hidden direct chat leaves the list, and the total with it.
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{direct_id}/users/{bob}/"),
            Some(bob)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = send_json!(app, unread(bob));
    assert_eq!(counts(&body), (0, 0), "{body}");
}

#[actix_web::test]
async fn unread_total_needs_a_session() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (status, _) = send!(app, request(Method::GET, "/api/v1/unread/", None));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
