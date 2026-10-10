use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::smart_db::SmartDatabase;
use std::fmt::Display;

/// Inserts a fresh user without any role: the seeded fixtures include administrators.
#[derive(serde::Serialize, serde::Deserialize)]
struct CreatePlainUser;

impl Display for CreatePlainUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CreatePlainUser")
    }
}

impl ApiRequestDto for CreatePlainUser {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO users (first_name, last_name, email, password, status) \
         VALUES ('Chat', 'Member', 'chat.member.' || gen_random_uuid() || '@mairie360.test', \
                 '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw', \
                 'active') \
         RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &[]
    }
}

pub async fn plain_user(db: &SmartDatabase) -> u64 {
    db.fetch_scalar::<i32, _>(&CreatePlainUser).await.unwrap() as u64
}

/// Inserts a fresh user without any role, named `first_name last_name` (names are what the chat
/// list searches and displays).
#[derive(serde::Serialize, serde::Deserialize)]
struct CreateNamedUser {
    params: Vec<QueryParam>,
}

impl Display for CreateNamedUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CreateNamedUser")
    }
}

impl ApiRequestDto for CreateNamedUser {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO users (first_name, last_name, email, password, status) \
         VALUES ($1, $2, 'chat.named.' || gen_random_uuid() || '@mairie360.test', \
                 '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw', \
                 'active') \
         RETURNING id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

pub async fn named_user(db: &SmartDatabase, first_name: &str, last_name: &str) -> u64 {
    let view = CreateNamedUser {
        params: vec![
            QueryParam::Text(first_name.to_string()),
            QueryParam::Text(last_name.to_string()),
        ],
    };
    db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64
}
