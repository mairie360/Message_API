//! The code under test connects as `message_api`, never as the superuser (MAIR-556): a superuser
//! ignores the grants of Database's `api_grants.sql`, so a query Message may not run in staging
//! would pass here.

use crate::common::{get_smart_db, RawQuery};
use serial_test::serial;

#[derive(serde::Serialize, serde::Deserialize)]
struct CurrentRole {
    name: String,
    is_superuser: bool,
}

#[tokio::test]
#[serial]
async fn test_mair_556_the_code_under_test_runs_as_message_api() {
    let db = get_smart_db().await;

    let role: CurrentRole = db
        .fetch_one(&RawQuery::new(
            "SELECT json_build_object('name', current_user::TEXT, 'is_superuser', rolsuper) \
             FROM pg_roles WHERE rolname = current_user",
            vec![],
        ))
        .await
        .unwrap();

    assert_eq!(role.name, "message_api");
    assert!(!role.is_superuser);
}

#[tokio::test]
#[serial]
async fn test_mair_556_message_api_is_held_to_its_grants() {
    // Message never reads the sessions of Core: the grants refuse it, as in staging.
    let db = get_smart_db().await;

    let error = db
        .fetch_scalar::<i64, _>(&RawQuery::new("SELECT count(*) FROM sessions", vec![]))
        .await
        .expect_err("message_api must not read sessions");

    assert!(
        error.to_string().contains("SQLSTATE 42501"),
        "expected insufficient_privilege, got {error}"
    );
}
