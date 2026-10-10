//! MAIR-507: the chat list carries everything it displays (name of a direct chat, member count) in
//! one page and can be searched by chat or member name; opening a chat returns its header and the
//! quoted messages; the members route returns names.

use actix_web::http::{Method, StatusCode};
use serde_json::{json, Value};
use serial_test::serial;
use std::sync::atomic::{AtomicU32, Ordering};

use super::harness::{admin_id, request, states};
use crate::common::{get_smart_db, named_user};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

/// A name no other test user has, so that searches only find what the test created (the test
/// database is shared and keeps every user).
fn unique(prefix: &str) -> String {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or_default();
    format!("{prefix}{nanos}x{}", NEXT.fetch_add(1, Ordering::SeqCst))
}

fn names(body: &Value) -> Vec<String> {
    body["chats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|chat| chat["name"].as_str().unwrap().to_string())
        .collect()
}

#[actix_web::test]
#[serial]
async fn list_names_a_direct_chat_after_the_other_participant_and_counts_members() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (last_a, last_b) = (unique("Martin"), unique("Bertrand"));
    let alice = named_user(&db, "Alice", &last_a).await;
    let bob = named_user(&db, "Xavier", &last_b).await;
    let carol = named_user(&db, "Carole", &unique("Durand")).await;

    let (_, direct) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": bob }))
    );
    let direct_id = direct["id"].as_u64().unwrap();
    let (_, group) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(alice))
            .set_json(json!({ "name": "Projet autoroute", "members": [bob, carol] }))
    );
    let group_id = group["id"].as_u64().unwrap();

    let (_, body) = send_json!(app, request(Method::GET, "/api/v1/", Some(alice)));
    let chats = body["chats"].as_array().unwrap();
    let find = |id: u64| chats.iter().find(|chat| chat["id"] == id).unwrap();
    assert_eq!(find(direct_id)["kind"], "direct");
    assert_eq!(find(direct_id)["name"], format!("Xavier {last_b}"));
    assert_eq!(find(direct_id)["contact_id"], bob);
    assert_eq!(find(direct_id)["member_count"], 2);
    assert_eq!(find(group_id)["name"], "Projet autoroute");
    assert_eq!(find(group_id)["member_count"], 3);

    // Bob sees the direct chat once a message was posted, under Alice's name.
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{direct_id}/messages/"),
            Some(alice)
        )
        .set_json(json!({ "content": "Bonjour" }))
    );
    assert_eq!(status, StatusCode::OK);
    let (_, body) = send_json!(app, request(Method::GET, "/api/v1/", Some(bob)));
    let seen = body["chats"]
        .as_array()
        .unwrap()
        .iter()
        .find(|chat| chat["id"] == direct_id)
        .unwrap();
    assert_eq!(seen["name"], format!("Alice {last_a}"));
    assert_eq!(seen["contact_id"], alice);
}

#[actix_web::test]
#[serial]
async fn search_finds_chats_by_their_name_or_by_a_member_name() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (last_a, last_b, last_c) = (unique("Martin"), unique("Bertrand"), unique("Durand"));
    let alice = named_user(&db, "Alice", &last_a).await;
    let xavier = named_user(&db, "Xavier", &last_b).await;
    let carole = named_user(&db, "Carole", &last_c).await;

    let title = unique("Autoroute");
    let mut ids = Vec::new();
    for (name, members) in [
        (title.as_str(), vec![carole]),
        ("Avec Xavier", vec![xavier, carole]),
        ("Sans Xavier", vec![carole]),
    ] {
        let (_, body) = send_json!(
            app,
            request(Method::POST, "/api/v1/", Some(alice))
                .set_json(json!({ "name": name, "members": members }))
        );
        ids.push(body["id"].as_u64().unwrap());
    }
    send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": xavier }))
    );

    // By the name of the chat, whatever the case.
    for term in [title.to_lowercase(), title.to_uppercase()] {
        let (status, body) = send_json!(
            app,
            request(Method::GET, &format!("/api/v1/?search={term}"), Some(alice))
        );
        assert_eq!(status, StatusCode::OK);
        assert_eq!(names(&body), vec![title.clone()], "{term}");
    }

    // By a member: first name, last name, both orders. The group without him and the caller's own
    // name find nothing more.
    let full = format!("Xavier {last_b}");
    let reversed = format!("{last_b} Xavier");
    for term in [last_b.as_str(), full.as_str(), reversed.as_str()] {
        let (_, body) = send_json!(
            app,
            request(
                Method::GET,
                &format!("/api/v1/?search={}", term.replace(' ', "%20")),
                Some(alice)
            )
        );
        let mut found = names(&body);
        found.sort();
        assert_eq!(
            found,
            vec!["Avec Xavier".to_string(), full.clone()],
            "{term}: the group he belongs to and the direct chat with him"
        );
    }
    let (_, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/?search={last_a}"),
            Some(alice)
        )
    );
    assert!(
        names(&body).is_empty(),
        "the caller's own name matches nothing: {body}"
    );

    // Wildcards of LIKE only match themselves.
    for term in ["%25", "_"] {
        let (_, body) = send_json!(
            app,
            request(Method::GET, &format!("/api/v1/?search={term}"), Some(alice))
        );
        assert!(names(&body).is_empty(), "{term}: {body}");
    }

    // Surrounding spaces are ignored; an empty search lists everything.
    let (_, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/?search=%20{}%20", title.to_lowercase()),
            Some(alice)
        )
    );
    assert_eq!(names(&body), vec![title.clone()]);
    let (_, body) = send_json!(app, request(Method::GET, "/api/v1/?search=", Some(alice)));
    assert_eq!(body["chats"].as_array().unwrap().len(), 4);
}

#[actix_web::test]
#[serial]
async fn search_only_sees_the_chats_of_the_caller_and_paginates_the_result() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let last = unique("Bertrand");
    let alice = named_user(&db, "Alice", &unique("Martin")).await;
    let stranger = named_user(&db, "Sam", &unique("Leroy")).await;
    let xavier = named_user(&db, "Xavier", &last).await;

    let mut mine = Vec::new();
    for name in ["Un", "Deux", "Trois"] {
        let (_, body) = send_json!(
            app,
            request(Method::POST, "/api/v1/", Some(alice))
                .set_json(json!({ "name": name, "members": [xavier] }))
        );
        mine.push(body["id"].as_u64().unwrap());
    }
    // A chat of Xavier the caller is not in must never come up.
    send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(stranger))
            .set_json(json!({ "name": "Prive", "members": [xavier] }))
    );

    let uri = format!("/api/v1/?search={last}&limit=2");
    let (_, body) = send_json!(app, request(Method::GET, &uri, Some(alice)));
    assert_eq!(names(&body), vec!["Trois", "Deux"]);
    assert_eq!(body["has_more"], true);
    let uri = format!("/api/v1/?search={last}&limit=2&offset=2");
    let (_, body) = send_json!(app, request(Method::GET, &uri, Some(alice)));
    assert_eq!(names(&body), vec!["Un"]);
    assert_eq!(body["has_more"], false);

    let (_, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/?search={last}"),
            Some(stranger)
        )
    );
    assert_eq!(names(&body), vec!["Prive"]);
    assert_eq!(mine.len(), 3);
}

#[actix_web::test]
#[serial]
async fn members_come_with_their_names() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (last_a, last_b) = (unique("Martin"), unique("Bertrand"));
    let alice = named_user(&db, "Alice", &last_a).await;
    let xavier = named_user(&db, "Xavier", &last_b).await;

    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(alice))
            .set_json(json!({ "name": "Equipe", "members": [xavier] }))
    );
    let chat_id = body["id"].as_u64().unwrap();
    let (status, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/users/"),
            Some(alice)
        )
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["users"],
        json!([
            { "id": alice, "first_name": "Alice", "last_name": last_a },
            { "id": xavier, "first_name": "Xavier", "last_name": last_b }
        ])
    );
}

#[actix_web::test]
#[serial]
async fn opening_a_chat_returns_its_header_and_the_quotes_with_their_excerpt() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (last_a, last_b) = (unique("Martin"), unique("Bertrand"));
    let alice = named_user(&db, "Alice", &last_a).await;
    let xavier = named_user(&db, "Xavier", &last_b).await;

    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": xavier }))
    );
    let chat_id = body["id"].as_u64().unwrap();
    let post = |user: u64, content: Value| {
        request(
            Method::POST,
            &format!("/api/v1/{chat_id}/messages/"),
            Some(user),
        )
        .set_json(content)
    };
    let long = "é".repeat(150);
    let (_, first) = send_json!(app, post(alice, json!({ "content": long })));
    let first_id = first["id"].as_u64().unwrap();
    let (_, second) = send_json!(
        app,
        post(xavier, json!({ "content": "Reçu", "citation": first_id }))
    );
    let second_id = second["id"].as_u64().unwrap();
    send_json!(app, post(alice, json!({ "content": "Merci" })));

    // Page of the 2 latest: the quoted message is older than the page and still comes with it.
    let (status, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/?limit=2"),
            Some(xavier)
        )
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["chat"]["id"], chat_id);
    assert_eq!(body["chat"]["kind"], "direct");
    // Xavier opened it: the name is the other participant's.
    assert_eq!(body["chat"]["name"], format!("Alice {last_a}"));
    assert_eq!(body["chat"]["contact_id"], alice);
    assert_eq!(body["chat"]["member_count"], 2);
    assert_eq!(body["chat"]["unread_count"], 2);
    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["id"], second_id);
    assert_eq!(messages[0]["citation"], first_id);
    assert_eq!(messages[0]["quoted"]["id"], first_id);
    assert_eq!(messages[0]["quoted"]["sender_id"], alice);
    assert_eq!(
        messages[0]["quoted"]["excerpt"]
            .as_str()
            .unwrap()
            .chars()
            .count(),
        100
    );
    assert!(messages[1]["quoted"].is_null());
    assert!(messages[1]["citation"].is_null());
}

#[actix_web::test]
#[serial]
async fn an_administrator_outside_a_direct_chat_sees_both_names() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (last_a, last_b) = (unique("Martin"), unique("Bertrand"));
    let alice = named_user(&db, "Alice", &last_a).await;
    let xavier = named_user(&db, "Xavier", &last_b).await;
    let admin = admin_id().await;

    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": xavier }))
    );
    let chat_id = body["id"].as_u64().unwrap();

    let (status, body) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(admin))
    );
    assert_eq!(status, StatusCode::OK);
    let name = body["chat"]["name"].as_str().unwrap();
    assert!(
        name.contains(&format!("Alice {last_a}")) && name.contains(&format!("Xavier {last_b}")),
        "{name}"
    );
    assert!(body["chat"]["contact_id"].is_null());
    assert_eq!(body["chat"]["unread_count"], 0);
}
