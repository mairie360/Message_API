//! MAIR-1228: `GET /api/v1/search`, Message's part of the global search of the shared header
//! (group chats and messages of the chats the caller belongs to). Each test uses a unique
//! marker as the term.

use actix_web::http::{Method, StatusCode};
use serde_json::{json, Value};

use super::harness::{admin_id, request, states};
use crate::common::{exclude_member, get_fixture_db, plain_user};

/// A marker made of letters only, unique to the test.
fn marker(prefix: &str) -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique = format!(
        "{nanos}{}",
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let letters: String = unique
        .bytes()
        .map(|digit| char::from(b'a' + (digit - b'0')))
        .collect();
    format!("{prefix}{letters}")
}

fn items(body: &Value) -> Vec<Value> {
    body["items"].as_array().expect("items").clone()
}

macro_rules! search {
    ($app:expr, $user:expr, $query:expr) => {{
        send_json!(
            $app,
            request(Method::GET, &format!("/api/v1/search{}", $query), $user)
        )
    }};
}

#[actix_web::test]
async fn mair_1228_finds_the_chat_and_its_messages_for_members_only() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let admin = admin_id().await;
    let (member, outsider) = (plain_user(&db).await, plain_user(&db).await);
    let term = marker("voirie");

    let (status, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(admin))
            .set_json(json!({ "name": format!("{term} équipe"), "members": [member] }))
    );
    assert_eq!(status, StatusCode::OK, "{body}");
    let chat = body["id"].as_u64().unwrap();
    let (status, body) = send_json!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat}/messages/"),
            Some(admin)
        )
        .set_json(json!({ "content": format!("Le marquage {term} est prévu jeudi") }))
    );
    assert_eq!(status, StatusCode::OK, "{body}");
    let message = body["id"].as_u64().unwrap();

    let (status, body) = search!(app, Some(member), format!("?q={term}"));
    assert_eq!(status, StatusCode::OK, "{body}");
    let found = items(&body);
    assert_eq!(found.len(), 2, "{body}");
    // The chat title starts with the term: it comes first.
    assert_eq!(found[0]["kind"], "conversation");
    assert_eq!(found[0]["id"], chat);
    assert_eq!(found[0]["title"], format!("{term} équipe"));
    assert!(found[0]["parent_id"].is_null() && found[0]["excerpt"].is_null());
    assert_eq!(found[1]["kind"], "message");
    assert_eq!(found[1]["id"], message);
    assert_eq!(found[1]["parent_id"], chat);
    assert_eq!(found[1]["title"], format!("{term} équipe"));
    assert_eq!(
        found[1]["excerpt"],
        format!("Le marquage {term} est prévu jeudi")
    );
    assert!(found[1]["updated_at"].as_str().unwrap().ends_with('Z'));

    let (_, body) = search!(app, Some(outsider), format!("?q={term}"));
    assert!(items(&body).is_empty(), "an outsider finds nothing: {body}");

    exclude_member(&db, chat, member).await;
    let (_, body) = search!(app, Some(member), format!("?q={term}"));
    assert!(
        items(&body).is_empty(),
        "an excluded member finds nothing: {body}"
    );
}

#[actix_web::test]
async fn mair_1228_limit_cuts_and_wildcards_match_themselves() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let admin = admin_id().await;
    let term = marker("budget");
    let (status, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(admin))
            .set_json(json!({ "name": "Finances", "members": [] }))
    );
    assert_eq!(status, StatusCode::OK, "{body}");
    let chat = body["id"].as_u64().unwrap();
    for index in 0..3 {
        let (status, _) = send_json!(
            app,
            request(
                Method::POST,
                &format!("/api/v1/{chat}/messages/"),
                Some(admin)
            )
            .set_json(json!({ "content": format!("Ligne {index} {term}") }))
        );
        assert_eq!(status, StatusCode::OK);
    }

    let (status, body) = search!(app, Some(admin), format!("?q={term}&limit=2"));
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(items(&body).len(), 2, "{body}");
    // Unescaped, `%_<term>` would match the rows above (any character before the term).
    let (_, body) = search!(app, Some(admin), format!("?q=%25_{term}"));
    assert!(items(&body).is_empty(), "{body}");
}

#[actix_web::test]
async fn mair_1228_rejects_a_short_term_a_bad_limit_and_no_session() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let admin = admin_id().await;
    for query in [
        "?q=a",
        "?q=%20a%20",
        "?q=%0Aab",
        "",
        "?q=ab&limit=21",
        "?q=ab&limit=0",
        "?q=ab&limit=abc",
        "?q=ab&limit=-1",
    ] {
        let (status, body) = send!(
            app,
            request(Method::GET, &format!("/api/v1/search{query}"), Some(admin))
        );
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}: {body}");
    }
    let (status, _) = send!(app, request(Method::GET, "/api/v1/search?q=voirie", None));
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
