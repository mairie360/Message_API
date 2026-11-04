//! Token refusals on every secured operation (MAIR-474), driven by the published contract
//! (`ApiDoc`): an operation is covered as soon as it is documented, and the sweep fails if one
//! accepts a token it should not.
//!
//! The tokens are the ones an attacker can build without Core's private key (MAIR-680): none at
//! all, another scheme, garbage, an HS256 token (no `JWT_SECRET` here, whatever secret signs it),
//! an `EdDSA` token signed with another key under Core's `kid`, one without `iss`/`aud` or for
//! another audience, an expired one, `alg: none`, a payload swapped under a valid signature, an
//! asymmetric algorithm (the Keycloak path, disabled here), plus well-signed tokens for an unknown
//! and for an archived account.

use actix_web::http::Method;
use actix_web::test::TestRequest;
use jsonwebtoken::{encode, EncodingKey, Header};
use mairie360_api_lib::jwt_manager::{Claims, SESSION_JWT_ISSUER};
use message_api::endpoints::swagger::ApiDoc;
use serial_test::serial;
use utoipa::OpenApi;

use actix_web::http::StatusCode;
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, BOB_ID};

use super::harness::{
    core_claims, now, other_private_key, request, sign_eddsa_with, sign_like_core, states, token,
};
use crate::common::{get_fixture_db, plain_user};

/// Every operation that declares the `jwt` scheme, as `(method, uri)`, path parameters set to `id`.
fn secured_operations(id: u64) -> Vec<(Method, String)> {
    let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let mut operations = Vec::new();
    for (template, methods) in document["paths"].as_object().unwrap() {
        let uri = template
            .split('/')
            .map(|segment| {
                if segment.starts_with('{') {
                    id.to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("/");
        for (method, operation) in methods.as_object().unwrap() {
            let secured = operation["security"]
                .as_array()
                .is_some_and(|schemes| schemes.iter().any(|scheme| scheme.get("jwt").is_some()));
            if secured {
                operations.push((
                    Method::from_bytes(method.to_uppercase().as_bytes()).unwrap(),
                    uri.clone(),
                ));
            }
        }
    }
    operations
}

/// Unpadded base64url, the JWT encoding of each part.
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, byte)| n | u32::from(*byte) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

/// Header values that must all be refused with `401`, labelled for the failure message.
fn refused_headers(user_id: u64) -> Vec<(&'static str, String)> {
    let valid = core_claims(user_id, now() + 3600);
    let hs256 = |claims: &Claims, key: &[u8]| {
        encode(&Header::default(), claims, &EncodingKey::from_secret(key)).unwrap()
    };
    let genuine = sign_like_core(&valid);
    let mut parts = genuine.split('.');
    let (header, signature) = (parts.next().unwrap(), parts.nth(1).unwrap());
    let other_user = base64url(
        serde_json::to_string(&core_claims(user_id + 1, now() + 3600))
            .unwrap()
            .as_bytes(),
    );
    let payload = base64url(serde_json::to_string(&valid).unwrap().as_bytes());

    vec![
        ("other scheme", format!("Basic {genuine}")),
        ("token without scheme", genuine.clone()),
        ("empty bearer", "Bearer ".to_string()),
        ("garbage", "Bearer not.a.jwt".to_string()),
        (
            "HS256 token, no JWT_SECRET on this API (MAIR-680)",
            format!(
                "Bearer {}",
                hs256(
                    &Claims::new(&user_id.to_string(), "User", now() + 3600),
                    b"endpoint-tests-only-jwt-secret-0123456789"
                )
            ),
        ),
        (
            "EdDSA signed with another key under Core's kid (MAIR-680)",
            format!("Bearer {}", sign_eddsa_with(&valid, &other_private_key())),
        ),
        (
            "EdDSA without iss nor aud (MAIR-680)",
            format!(
                "Bearer {}",
                sign_like_core(&Claims::new(&user_id.to_string(), "User", now() + 3600))
            ),
        ),
        (
            "EdDSA for another audience (MAIR-680)",
            format!(
                "Bearer {}",
                sign_like_core(
                    &Claims::new(&user_id.to_string(), "User", now() + 3600)
                        .with_issuer_and_audience(SESSION_JWT_ISSUER, "another-platform")
                )
            ),
        ),
        (
            "expired",
            format!(
                "Bearer {}",
                sign_like_core(&core_claims(user_id, now() - 60))
            ),
        ),
        (
            "alg none",
            format!(
                "Bearer {}.{payload}.",
                base64url(br#"{"alg":"none","typ":"JWT"}"#)
            ),
        ),
        (
            "payload swapped under a valid signature",
            format!("Bearer {header}.{other_user}.{signature}"),
        ),
        (
            "asymmetric algorithm",
            format!(
                "Bearer {}.{payload}.{signature}",
                base64url(br#"{"alg":"RS256","typ":"JWT"}"#)
            ),
        ),
    ]
}

#[actix_web::test]
#[serial]
async fn every_secured_operation_refuses_a_missing_or_forged_token() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let user = plain_user(&get_fixture_db().await).await;
    // Sets the JWT environment (Core's public key only) the tokens are checked against.
    let _ = token(user);

    let operations = secured_operations(1);
    assert!(operations.len() >= 12, "{operations:?}");
    let headers = refused_headers(user);

    for (method, uri) in &operations {
        let (status, _) = send!(app, TestRequest::default().method(method.clone()).uri(uri));
        assert_eq!(status, StatusCode::UNAUTHORIZED, "no token: {method} {uri}");

        for (label, value) in &headers {
            let forged = TestRequest::default()
                .method(method.clone())
                .uri(uri)
                .insert_header(("Authorization", value.as_str()))
                .set_json(serde_json::json!({}));
            let (status, _) = send!(app, forged);
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{label}: {method} {uri}");
        }
    }
}

#[actix_web::test]
#[serial]
async fn every_secured_operation_refuses_an_unknown_or_archived_account() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    get_shared_db().await;
    // Bob is seeded archived by the lib's test setup.
    let archived = u64::try_from(*BOB_ID.get().expect("Bob ID missing")).unwrap();

    for (method, uri) in secured_operations(1) {
        for (label, holder) in [
            ("unknown", u64::from(i32::MAX.unsigned_abs())),
            ("archived", archived),
        ] {
            let (status, _) = send!(
                app,
                request(method.clone(), &uri, Some(holder)).set_json(serde_json::json!({}))
            );
            assert_eq!(status, StatusCode::NOT_FOUND, "{label}: {method} {uri}");
        }
    }
}

/// Control of the sweeps: a genuine token of an active account passes the authentication of every
/// operation (whatever the handler answers next), so their `401`s come from the token alone.
#[actix_web::test]
#[serial]
async fn a_genuine_token_passes_the_authentication_of_every_operation() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let user = plain_user(&get_fixture_db().await).await;

    for (method, uri) in secured_operations(1) {
        // The stream never ends: its status is read without waiting for the body.
        if uri.ends_with("/stream") {
            let response = actix_web::test::call_service(
                &app,
                request(method.clone(), &uri, Some(user)).to_request(),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK, "{method} {uri}");
            continue;
        }
        let (status, _) = send!(
            app,
            request(method.clone(), &uri, Some(user)).set_json(serde_json::json!({}))
        );
        assert!(
            status != StatusCode::UNAUTHORIZED && !status.is_server_error(),
            "{status}: {method} {uri}"
        );
    }
}

/// The sweeps above only see the operations that declare `jwt`: an operation published under
/// `/api` without it would be skipped silently, so every one of them must declare it.
#[test]
fn every_api_operation_declares_jwt() {
    let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let mut unsecured = Vec::new();
    for (template, methods) in document["paths"].as_object().unwrap() {
        if !template.starts_with("/api/") {
            continue;
        }
        for (method, operation) in methods.as_object().unwrap() {
            let secured = operation["security"]
                .as_array()
                .is_some_and(|schemes| schemes.iter().any(|scheme| scheme.get("jwt").is_some()));
            if !secured {
                unsecured.push(format!("{} {template}", method.to_uppercase()));
            }
        }
    }
    assert!(
        unsecured.is_empty(),
        "published without `jwt`: {unsecured:?}"
    );
}
