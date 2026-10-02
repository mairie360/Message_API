//! Swagger UI and the OpenAPI document are only served on demand (MAIR-424).

use actix_web::http::{Method, StatusCode};
use message_api::endpoints::swagger::docs_config;

use super::harness::request;

#[actix_web::test]
async fn docs_are_not_served_unless_enabled() {
    for enabled in [false, true] {
        let app = actix_web::test::init_service(
            actix_web::App::new().configure(|cfg| docs_config(cfg, enabled)),
        )
        .await;
        let expected = if enabled {
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        };
        for uri in ["/api-docs/openapi.json", "/swagger-ui/"] {
            let (status, _) = send!(app, request(Method::GET, uri, None));
            assert_eq!(status, expected, "{uri} with SWAGGER_ENABLED={enabled}");
        }
    }
}

#[actix_web::test]
async fn the_template_hello_route_is_gone() {
    let document = serde_json::to_value(
        <message_api::endpoints::swagger::ApiDoc as utoipa::OpenApi>::openapi(),
    )
    .unwrap();
    assert!(document["paths"].get("/").is_none());
}
