//! A path id that is not an integer answers the documented `400 text/plain` on every operation
//! (MAIR-262): the `DELETE` routes answered actix's default `404` while the contract documents
//! `400`.
//!
//! Driven by the published contract (`ApiDoc`), so a new operation with a path id is covered as
//! soon as it is documented.

use actix_web::http::{header::CONTENT_TYPE, Method, StatusCode};
use message_api::endpoints::swagger::ApiDoc;
use serde_json::json;
use serial_test::serial;
use utoipa::OpenApi;

use super::harness::{admin_id, request, states};

/// The documented invalid-path answers (they never echo the value).
const INVALID_PATH_BODIES: &[&str] = &["Invalid path: ids are integers."];

/// Every published operation with a path parameter, as `(method, uri, documented statuses)`,
/// once per path parameter, replaced by `abc` (the other ids by `1`).
fn operations_with_a_path_id() -> Vec<(Method, String, Vec<String>)> {
    let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let mut operations = Vec::new();
    for (template, methods) in document["paths"].as_object().unwrap() {
        if !template.contains('{') {
            continue;
        }
        let segments: Vec<&str> = template.split('/').collect();
        let ids = segments.iter().filter(|s| s.starts_with('{')).count();
        for (method, operation) in methods.as_object().unwrap() {
            let statuses: Vec<String> = operation["responses"]
                .as_object()
                .map(|responses| responses.keys().cloned().collect())
                .unwrap_or_default();
            // One request per path id: that id is `abc`, the others a valid integer, so a `400`
            // caused by another id cannot satisfy the check.
            for invalid in 0..ids {
                let mut index = 0;
                let uri = segments
                    .iter()
                    .map(|segment| {
                        if segment.starts_with('{') {
                            index += 1;
                            if index - 1 == invalid {
                                "abc"
                            } else {
                                "1"
                            }
                        } else {
                            segment
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("/");
                operations.push((
                    Method::from_bytes(method.to_uppercase().as_bytes()).unwrap(),
                    uri,
                    statuses.clone(),
                ));
            }
        }
    }
    operations
}

#[actix_web::test]
#[serial]
async fn a_path_id_that_is_not_an_integer_answers_the_documented_400() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let admin = admin_id().await;

    let operations = operations_with_a_path_id();
    assert!(operations.len() >= 8, "{operations:?}");
    let mut failures = Vec::new();
    for (method, uri, documented) in operations {
        if !documented.iter().any(|status| status == "400") {
            failures.push(format!("{method} {uri}: 400 is not documented"));
        }
        // A valid body, so that only the path can be refused.
        let req = request(method.clone(), &uri, Some(admin))
            .set_json(json!({ "content": "abc", "users_id": [admin], "readUntilMessageId": 1 }));
        let response = match actix_web::test::try_call_service(&app, req.to_request()).await {
            Ok(response) => response.map_into_boxed_body(),
            Err(error) => actix_web::dev::ServiceResponse::new(
                actix_web::test::TestRequest::default().to_http_request(),
                error.error_response(),
            ),
        };
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        let status = response.status();
        let body = actix_web::test::read_body(response).await;
        let body = String::from_utf8_lossy(&body);
        if status != StatusCode::BAD_REQUEST || !content_type.starts_with("text/plain") {
            failures.push(format!("{method} {uri}: {status} {content_type} {body}"));
        } else if !INVALID_PATH_BODIES.contains(&body.as_ref()) {
            // The exact message, so that another `400` (a body refused) cannot pass.
            failures.push(format!("{method} {uri}: unexpected body: {body}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
