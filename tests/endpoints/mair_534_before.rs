//! MAIR-534: the `before` cursor of `GET /{id}/` is bounded (1 to i64::MAX): `0` and an overflowing value are a 400,
//! the nominal cursor still pages.

use actix_web::http::{Method, StatusCode};
use serde_json::json;

use super::harness::{request, states};
use crate::common::{get_fixture_db, plain_user};

#[actix_web::test]
async fn before_out_of_range_is_a_400_and_a_valid_cursor_still_pages() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let user = plain_user(&db).await;
    let other = plain_user(&db).await;
    let (_, chat) = send_json!(
        app,
        request(Method::POST, "/api/v1/", Some(user))
            .set_json(json!({ "name": "Cursor", "members": [other] }))
    );
    let chat_id = chat["id"].as_u64().unwrap();

    for before in ["0", "9223372036854775808", "-1", "abc"] {
        let (status, _) = send!(
            app,
            request(
                Method::GET,
                &format!("/api/v1/{chat_id}/?before={before}"),
                Some(user)
            )
        );
        assert_eq!(status, StatusCode::BAD_REQUEST, "before={before}");
    }
    let (status, _) = send!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat_id}/?before=9223372036854775807"),
            Some(user)
        )
    );
    assert_eq!(status, StatusCode::OK);
}
