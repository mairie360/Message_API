//! Who may not call what: every refusal of the routes, checked through the real middleware.

use actix_web::http::{Method, StatusCode};
use serde_json::json;
use serial_test::serial;

use super::harness::{admin_id, request, states};
use crate::common::{get_smart_db, plain_user};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

#[actix_web::test]
#[serial]
async fn every_api_route_requires_a_valid_jwt() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);

    for (method, uri) in [
        (Method::GET, "/api/v1/"),
        (Method::POST, "/api/v1/"),
        (Method::GET, "/api/v1/stream"),
        (Method::GET, "/api/v1/1/"),
        (Method::DELETE, "/api/v1/1/"),
        (Method::POST, "/api/v1/1/messages/"),
        (Method::PATCH, "/api/v1/1/messages/1/"),
        (Method::DELETE, "/api/v1/1/messages/1/"),
        (Method::POST, "/api/v1/1/read/"),
        (Method::GET, "/api/v1/1/users/"),
        (Method::POST, "/api/v1/1/users/"),
        (Method::DELETE, "/api/v1/1/users/1/"),
    ] {
        let (status, _) = send!(app, request(method.clone(), uri, None));
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
        let (status, _) = send!(
            app,
            request(method.clone(), uri, None).insert_header(("Authorization", "Bearer not.a.jwt"))
        );
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri} (forged)");
    }
}

/// A chat of `creator` and `member` with one message of each; returns (chat, creator's message,
/// member's message).
macro_rules! seeded_chat {
    ($app:expr, $creator:expr, $member:expr) => {{
        let (_, body) = send_json!(
            $app,
            request(Method::POST, "/api/v1/", Some($creator))
                .set_json(json!({ "name": "Astreinte", "members": [$member] }))
        );
        let chat_id = body["id"].as_u64().unwrap();
        let mut ids = Vec::new();
        for author in [$creator, $member] {
            let (_, body) = send_json!(
                $app,
                request(Method::POST, &format!("/api/v1/{chat_id}/messages/"), Some(author))
                    .set_json(json!({ "content": "Message" }))
            );
            ids.push(body["id"].as_u64().unwrap());
        }
        (chat_id, ids[0], ids[1])
    }};
}

#[actix_web::test]
#[serial]
async fn outsiders_get_the_same_404_as_for_an_unknown_chat() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (creator, member, outsider) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let (chat, message, _) = seeded_chat!(app, creator, member);

    let calls = [
        (Method::GET, format!("/api/v1/{chat}/"), None),
        (Method::DELETE, format!("/api/v1/{chat}/"), None),
        (
            Method::POST,
            format!("/api/v1/{chat}/messages/"),
            Some(json!({ "content": "Intrus" })),
        ),
        (
            Method::PATCH,
            format!("/api/v1/{chat}/messages/{message}/"),
            Some(json!({ "content": "Intrus" })),
        ),
        (
            Method::DELETE,
            format!("/api/v1/{chat}/messages/{message}/"),
            None,
        ),
        (
            Method::POST,
            format!("/api/v1/{chat}/read/"),
            Some(json!({ "readUntilMessageId": message })),
        ),
        (Method::GET, format!("/api/v1/{chat}/users/"), None),
        (
            Method::POST,
            format!("/api/v1/{chat}/users/"),
            Some(json!({ "users_id": [outsider] })),
        ),
        (
            Method::DELETE,
            format!("/api/v1/{chat}/users/{member}/"),
            None,
        ),
    ];
    for (method, uri, body) in calls {
        for target in [
            uri.clone(),
            uri.replacen(&chat.to_string(), "2147483000", 1),
        ] {
            let mut call = request(method.clone(), &target, Some(outsider));
            if let Some(body) = &body {
                call = call.set_json(body);
            }
            let (status, _) = send!(app, call);
            assert_eq!(status, StatusCode::NOT_FOUND, "{method} {target}");
        }
    }

    // Nothing was changed by the refused calls.
    let (_, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat}/users/"),
            Some(creator)
        )
    );
    assert_eq!(body["users"].as_array().unwrap().len(), 2);
    let (_, body) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat}/"), Some(creator))
    );
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);
}

#[actix_web::test]
#[serial]
async fn url_encoded_chat_id_is_checked_like_the_plain_one() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (creator, member, outsider) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let (chat, _, _) = seeded_chat!(app, creator, member);
    // Every digit percent-encoded: `12` → `%31%32`.
    let encoded: String = chat
        .to_string()
        .bytes()
        .map(|b| format!("%{b:02X}"))
        .collect();

    let (status, _) = send!(
        app,
        request(Method::GET, &format!("/api/v1/{encoded}/"), Some(outsider))
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send!(
        app,
        request(Method::GET, &format!("/api/v1/{encoded}/"), Some(member))
    );
    assert_eq!(status, StatusCode::OK);
}

#[actix_web::test]
#[serial]
async fn members_only_manage_themselves_and_their_own_messages() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (creator, member, outsider) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let (chat, creators_message, _) = seeded_chat!(app, creator, member);

    let forbidden = [
        (
            Method::POST,
            format!("/api/v1/{chat}/users/"),
            Some(json!({ "users_id": [outsider] })),
        ),
        (
            Method::DELETE,
            format!("/api/v1/{chat}/users/{creator}/"),
            None,
        ),
        (Method::DELETE, format!("/api/v1/{chat}/"), None),
        (
            Method::PATCH,
            format!("/api/v1/{chat}/messages/{creators_message}/"),
            Some(json!({ "content": "Réécrit" })),
        ),
        (
            Method::DELETE,
            format!("/api/v1/{chat}/messages/{creators_message}/"),
            None,
        ),
    ];
    for (method, uri, body) in forbidden {
        let mut call = request(method.clone(), &uri, Some(member));
        if let Some(body) = body {
            call = call.set_json(body);
        }
        let (status, _) = send!(app, call);
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {uri}");
    }

    // A member who left can no longer post nor read.
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat}/users/{member}/"),
            Some(member)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat}/messages/"),
            Some(member)
        )
        .set_json(json!({ "content": "Encore là ?" }))
    );
    assert_eq!(status, StatusCode::NOT_FOUND);

    // The creator who left loses the management rights too.
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat}/users/{creator}/"),
            Some(creator)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat}/users/"),
            Some(creator)
        )
        .set_json(json!({ "users_id": [outsider] }))
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[actix_web::test]
#[serial]
async fn administrators_moderate_but_do_not_take_part() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (creator, member) = (plain_user(&db).await, plain_user(&db).await);
    let admin = admin_id().await;
    let (chat, message, _) = seeded_chat!(app, creator, member);

    let (status, body) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat}/messages/"),
            Some(admin)
        )
        .set_json(json!({ "content": "Bonjour" }))
    );
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body.contains("Administrators"));

    let (status, _) = send!(
        app,
        request(
            Method::PATCH,
            &format!("/api/v1/{chat}/messages/{message}/"),
            Some(admin)
        )
        .set_json(json!({ "content": "Réécrit" }))
    );
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = send!(
        app,
        request(Method::GET, &format!("/api/v1/{chat}/users/"), Some(admin))
    );
    assert_eq!(status, StatusCode::OK);
}

#[actix_web::test]
#[serial]
async fn ids_from_another_chat_are_refused() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let (author, other) = (plain_user(&db).await, plain_user(&db).await);
    let (chat_a, _, _) = seeded_chat!(app, author, other);
    let (_, message_b, _) = seeded_chat!(app, author, other);

    // The author of `message_b` is a member of both chats: only the chat of the path counts.
    let (status, _) = send!(
        app,
        request(
            Method::PATCH,
            &format!("/api/v1/{chat_a}/messages/{message_b}/"),
            Some(author)
        )
        .set_json(json!({ "content": "Mauvaise conversation" }))
    );
    // Documented as `400 Unknown event.` for an unknown message of the chat.
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_a}/messages/{message_b}/"),
            Some(author)
        )
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_a}/read/"),
            Some(author)
        )
        .set_json(json!({ "readUntilMessageId": message_b }))
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat_a}/messages/"),
            Some(author)
        )
        .set_json(json!({ "content": "Réponse", "citation": message_b }))
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[actix_web::test]
#[serial]
async fn invalid_bodies_are_refused_before_any_write() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let user = plain_user(&db).await;

    for body in [
        json!({ "name": "a".repeat(151), "members": [] }),
        json!({ "name": "Doublons", "members": [user, user] }),
        json!({ "name": "Inconnu", "members": [2147483000] }),
        json!({ "name": "Hors borne", "members": [4294967297u64] }),
        json!({ "members": [] }),
    ] {
        let (status, _) = send!(
            app,
            request(Method::POST, "/api/v1/", Some(user)).set_json(&body)
        );
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }

    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(user))
            .set_json(json!({ "name": "", "members": [] }))
    );
    let chat = body["id"].as_u64().unwrap();
    for (method, uri, body) in [
        (
            Method::POST,
            format!("/api/v1/{chat}/messages/"),
            json!({ "content": "   " }),
        ),
        (
            Method::POST,
            format!("/api/v1/{chat}/messages/"),
            json!({ "content": "a".repeat(5001) }),
        ),
        (
            Method::POST,
            format!("/api/v1/{chat}/read/"),
            json!({ "readUntilMessageId": 0 }),
        ),
        (
            Method::POST,
            format!("/api/v1/{chat}/users/"),
            json!({ "users_id": [] }),
        ),
    ] {
        let (status, _) = send!(
            app,
            request(method.clone(), &uri, Some(user)).set_json(&body)
        );
        assert_eq!(status, StatusCode::BAD_REQUEST, "{method} {uri} {body}");
    }
    let (status, _) = send!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat}/?limit=101"),
            Some(user)
        )
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
