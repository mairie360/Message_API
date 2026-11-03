//! MAIR-680: Core is the only signer of the session JWTs. This API holds Core's public key
//! (`JWT_PUBLIC_KEY`) and no `JWT_SECRET`: it accepts the `EdDSA` tokens signed with Core's key and
//! refuses every HS256 token, whatever secret signed it, so a service that leaks its configuration
//! can no longer forge an admin token for this API.

use actix_web::http::{Method, StatusCode};
use actix_web::test::TestRequest;
use jsonwebtoken::{encode, EncodingKey, Header};
use mairie360_api_lib::jwt_manager::Claims;
use serde_json::json;
use serial_test::serial;

use super::harness::{admin_id, now, request, set_jwt_env, states};
use crate::common::{get_fixture_db, plain_user};

/// HS256 token of `user_id` signed with `secret`: what any holder of the old shared secret signs.
fn hs256(user_id: u64, secret: &[u8]) -> TestRequest {
    let token = encode(
        &Header::default(),
        &Claims::new(&user_id.to_string(), "admin", now() + 3600),
        &EncodingKey::from_secret(secret),
    )
    .unwrap();
    TestRequest::default().insert_header(("Authorization", format!("Bearer {token}")))
}

#[actix_web::test]
#[serial]
async fn mair_680_runs_without_jwt_secret() {
    set_jwt_env();
    assert!(std::env::var_os("JWT_SECRET").is_none());
    assert!(std::env::var_os("JWT_PRIVATE_KEY").is_none());
    assert!(std::env::var_os("JWT_PUBLIC_KEY").is_some());
}

#[actix_web::test]
#[serial]
async fn mair_680_eddsa_token_signed_with_core_key_is_accepted() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let user = plain_user(&get_fixture_db().await).await;

    let (status, _) = send!(app, request(Method::GET, "/api/v1/", Some(user)));
    assert_eq!(status, StatusCode::OK);
}

#[actix_web::test]
#[serial]
async fn mair_680_hs256_token_is_refused_without_jwt_secret() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let admin = admin_id().await;

    // The secrets the stacks and the test suites used to share, and any other one.
    for secret in [
        b"endpoint-tests-only-jwt-secret-0123456789".as_slice(),
        b"b\"secret\"",
        b"any-secret-of-a-compromised-service-0123456789",
    ] {
        for uri in ["/api/v1/", "/api/v1/stream"] {
            let (status, body) = send!(app, hs256(admin, secret).method(Method::GET).uri(uri));
            assert_eq!(status, StatusCode::UNAUTHORIZED, "HS256 on {uri}: {body}");
        }
    }
}

/// Business journey with Core's tokens: an agent opens a group chat with a colleague, both write
/// in it, and the creator deletes it; the creator's HS256 token is refused on that chat.
#[actix_web::test]
#[serial]
async fn mair_680_agents_chat_with_core_tokens_only() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let creator = plain_user(&db).await;
    let member = plain_user(&db).await;

    let (status, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(creator))
            .set_json(json!({ "name": "Astreinte voirie", "members": [member] }))
    );
    assert_eq!(status, StatusCode::OK);
    let chat_id = body["id"].as_u64().unwrap();
    let messages = format!("/api/v1/{chat_id}/messages/");

    let (status, _) = send_json!(
        app,
        request(Method::POST, &messages, Some(member))
            .set_json(json!({ "content": "Je prends l'astreinte samedi." }))
    );
    assert_eq!(status, StatusCode::OK);

    let (status, body) = send!(
        app,
        request(Method::GET, &format!("/api/v1/{chat_id}/"), Some(creator))
    );
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("astreinte samedi"), "{body}");

    for (method, uri) in [
        (Method::GET, format!("/api/v1/{chat_id}/")),
        (Method::DELETE, format!("/api/v1/{chat_id}/")),
    ] {
        let (status, body) = send!(app, hs256(creator, b"b\"secret\"").method(method).uri(&uri));
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{uri}: {body}");
        assert!(!body.contains("astreinte samedi"));
    }

    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat_id}/"),
            Some(creator)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
}
