//! MAIR-288: `access-matrix.yaml` covers every operation of the `OpenAPI`, and the API answers as
//! it says. Each operation is called on a group chat without a session, as a user of each role who
//! has no relation to it, and as its creator, a member and the author of a message: an allowed
//! caller never gets 401 / 403 / 404, any other caller gets 403, or 404 when the chat is hidden.

use std::collections::{BTreeMap, BTreeSet};

use std::fmt::Display;

use actix_web::http::Method;
use actix_web::test::TestRequest;
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::smart_db::SmartDatabase;
use message_api::endpoints::swagger::ApiDoc;
use serde::Deserialize;
use serde_json::{json, Value};
use serial_test::serial;
use utoipa::OpenApi;

use super::harness::{admin_id, request as as_caller, states};
use crate::common::{get_smart_db, plain_user};
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;

const MATRIX: &str = include_str!("../../access-matrix.yaml");
const ROLES: [&str; 5] = ["Admin", "Maire", "Responsable", "User", "Guest"];
const RELATIONS: [&str; 3] = ["creator", "member", "author"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Matrix {
    version: u32,
    api: String,
    roles: Vec<String>,
    operations: BTreeMap<String, Operation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Operation {
    access: String,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    ownership: Vec<String>,
    #[serde(default)]
    personal: Vec<String>,
    #[allow(dead_code)]
    note: Option<String>,
}

fn matrix() -> Matrix {
    yaml_serde::from_str(MATRIX).expect("access-matrix.yaml is valid")
}

/// The spec the API serves (`ApiDoc`, the contract the BFF is generated from), as JSON.
fn spec() -> Value {
    serde_json::to_value(ApiDoc::openapi()).unwrap()
}

/// "METHOD /path" of every operation of the spec.
fn spec_operations(spec: &Value) -> BTreeSet<String> {
    let mut operations = BTreeSet::new();
    for (path, item) in spec["paths"].as_object().unwrap() {
        for method in ["get", "post", "put", "patch", "delete"] {
            if item.get(method).is_some() {
                operations.insert(format!("{} {path}", method.to_uppercase()));
            }
        }
    }
    operations
}

#[test]
fn the_matrix_covers_every_operation_of_the_spec() {
    let matrix = matrix();
    assert_eq!((matrix.version, matrix.api.as_str()), (1, "message"));
    assert_eq!(matrix.roles, ROLES);
    let spec = spec_operations(&spec());
    let listed: BTreeSet<String> = matrix.operations.keys().cloned().collect();
    let missing: Vec<_> = spec.difference(&listed).collect();
    let unknown: Vec<_> = listed.difference(&spec).collect();
    assert!(
        missing.is_empty(),
        "operations of the OpenAPI missing from access-matrix.yaml: {missing:?}"
    );
    assert!(
        unknown.is_empty(),
        "operations of access-matrix.yaml that the OpenAPI does not have: {unknown:?}"
    );
    for (name, op) in &matrix.operations {
        assert!(
            ["public", "authenticated"].contains(&op.access.as_str()),
            "{name}: access"
        );
        assert!(
            op.roles.iter().all(|r| ROLES.contains(&r.as_str())),
            "{name}: unknown role"
        );
        assert!(
            op.ownership.iter().all(|o| RELATIONS.contains(&o.as_str())),
            "{name}: ownership"
        );
        if op.access == "authenticated" {
            assert!(
                !op.roles.is_empty() || !op.ownership.is_empty(),
                "{name}: nobody may call it"
            );
        }
        assert!(
            op.personal.iter().all(|f| !f.is_empty()),
            "{name}: personal"
        );
    }
}

/// Gives `user` the role `name` (the schema also gives every account the Guest role).
#[derive(serde::Serialize, serde::Deserialize)]
struct GrantRole {
    params: Vec<QueryParam>,
}

impl Display for GrantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GrantRole")
    }
}

impl ApiRequestDto for GrantRole {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE name = $2 \
         ON CONFLICT DO NOTHING"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

async fn user_with_role(db: &SmartDatabase, role: Option<&str>) -> u64 {
    let id = plain_user(db).await;
    if let Some(role) = role {
        db.execute(GrantRole {
            params: vec![
                QueryParam::I32(i32::try_from(id).unwrap()),
                QueryParam::Text(role.to_string()),
            ],
        })
        .await
        .unwrap();
    }
    id
}

/// The status only: the body of `GET /stream` never ends (server-sent events).
macro_rules! status_only {
    ($app:expr, $request:expr) => {{
        match actix_web::test::try_call_service(&$app, $request.to_request()).await {
            Ok(response) => response.status().as_u16(),
            Err(error) => error.as_response_error().status_code().as_u16(),
        }
    }};
}

/// The people around one chat, created once.
struct People {
    creator: u64,
    member: u64,
    author: u64,
}

/// A fresh group chat of the creator with the member, the author and an extra member, and a
/// message of the author.
struct Fixtures {
    chat: u64,
    message: u64,
    extra: u64,
}

macro_rules! fixtures {
    ($app:expr, $db:expr, $people:expr) => {{
        let extra = user_with_role($db, Some("User")).await;
        let (status, created) = send_json!(
            $app,
            as_caller(Method::POST, "/api/v1/", Some($people.creator)).set_json(
                json!({ "name": "Matrice d'accès", "members": [$people.member, $people.author, extra] })
            )
        );
        assert!(status.is_success(), "fixture: create the chat: {status}");
        let chat = created["id"].as_u64().unwrap();
        let (status, posted) = send_json!(
            $app,
            as_caller(
                Method::POST,
                &format!("/api/v1/{chat}/messages/"),
                Some($people.author)
            )
            .set_json(json!({ "content": "Message de la matrice" }))
        );
        assert!(status.is_success(), "fixture: post the message: {status}");
        Fixtures {
            chat,
            message: posted["id"].as_u64().unwrap(),
            extra,
        }
    }};
}

/// The request of one operation on the fixtures: path parameters, and the example body of the
/// spec, made valid for the fixtures (no foreign ids).
fn request(
    spec: &Value,
    name: &str,
    f: &Fixtures,
    caller: Option<u64>,
    joiner: u64,
) -> TestRequest {
    let (method, path) = name.split_once(' ').unwrap();
    let uri = path
        .replace("{chat_id}", &f.chat.to_string())
        .replace("{message_id}", &f.message.to_string())
        .replace("{user_id}", &f.extra.to_string());
    let example = spec["paths"][path][method.to_lowercase()]["requestBody"]["content"]
        ["application/json"]["example"]
        .clone();
    let body = match name {
        "POST /api/v1/" => Some(json!({ "name": "Nouvelle discussion", "members": [joiner] })),
        "POST /api/v1/direct/" => Some(json!({ "contact_id": joiner })),
        "POST /api/v1/{chat_id}/messages/" => Some(json!({ "content": "Bonjour" })),
        "POST /api/v1/{chat_id}/read/" => Some(json!({ "readUntilMessageId": f.message })),
        "POST /api/v1/{chat_id}/users/" => Some(json!({ "users_id": [joiner] })),
        _ if example.is_null() => None,
        _ => Some(example),
    };
    let request = as_caller(Method::from_bytes(method.as_bytes()).unwrap(), &uri, caller);
    match body {
        Some(body) => request.set_json(body),
        None => request,
    }
}

#[actix_web::test]
#[serial]
async fn every_role_and_relation_gets_what_the_matrix_says() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let (_container, url) = get_shared_db().await;
    let db = get_smart_db(url).await;
    let spec = spec();
    let people = People {
        creator: user_with_role(&db, Some("User")).await,
        member: user_with_role(&db, Some("User")).await,
        author: user_with_role(&db, Some("User")).await,
    };
    // Users of each role with no relation to the chats.
    let mut callers: Vec<(String, u64)> = vec![("Admin".to_string(), admin_id().await)];
    for role in &ROLES[1..] {
        let id = user_with_role(&db, (*role != "Guest").then_some(*role)).await;
        callers.push(((*role).to_string(), id));
    }

    let mut failures = Vec::new();
    for (name, op) in &matrix().operations {
        if op.access == "public" {
            continue;
        }
        let f = fixtures!(app, &db, people);
        let anonymous = status_only!(app, request(&spec, name, &f, None, f.extra));
        if anonymous != 401 {
            failures.push(format!(
                "{name}: got {anonymous} without a session, expected 401"
            ));
        }
        let mut cases: Vec<(String, u64, bool)> = callers
            .iter()
            .map(|(role, id)| (role.clone(), *id, op.roles.contains(role)))
            .collect();
        for (who, user) in [
            ("creator", people.creator),
            ("member", people.member),
            ("author", people.author),
        ] {
            let allowed =
                op.roles.iter().any(|r| r == "User") || op.ownership.iter().any(|o| o == who);
            cases.push((who.to_string(), user, allowed));
        }
        for (who, user, allowed) in cases {
            let f = fixtures!(app, &db, people);
            let joiner = user_with_role(&db, Some("User")).await;
            let got = status_only!(app, request(&spec, name, &f, Some(user), joiner));
            let refused = got == 401 || got == 403 || got == 404;
            if allowed && refused {
                failures.push(format!(
                    "{name}: {who} is allowed by the matrix but got {got}"
                ));
            }
            if !allowed && got != 403 && got != 404 {
                failures.push(format!(
                    "{name}: {who} is not allowed by the matrix but got {got}, expected 403 or 404"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "the API does not answer as access-matrix.yaml says:\n{}",
        failures.join("\n")
    );
}
