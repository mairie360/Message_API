//! Input validation shared by every request body and query string of the API.
//!
//! A request view implements [`Validate`] and the handler extracts it with [`ValidatedJson`] or
//! [`ValidatedQuery`] instead of `web::Json` / `web::Query`: an invalid value is rejected with a
//! `400 Bad Request` (plain-text body naming the field) before the handler runs, so it never
//! reaches Postgres (where an over-long value or a NUL byte used to end in a `500`) nor comes back
//! unescaped in a JSON response.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use serde::de::DeserializeOwned;

/// `conversations.title` is `VARCHAR(150)`.
pub const MAX_TITLE_LENGTH: usize = 150;
/// `messages.content` is `TEXT`, capped to keep the payloads reasonable.
pub const MAX_MESSAGE_LENGTH: usize = 5000;
/// Users attached to a chat in one request (chat creation or `POST /{chat_id}/users/`).
pub const MAX_MEMBERS_PER_REQUEST: usize = 50;
/// User ids are `INTEGER` in Postgres: anything above would fail at the database.
pub const MAX_USER_ID: u64 = i32::MAX as u64;
/// Message ids are `BIGINT` in Postgres.
pub const MAX_MESSAGE_ID: u64 = i64::MAX as u64;

/// Why a request value was rejected; its text is the body of the `400` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(String);

impl ValidationError {
    pub fn new(field: &str, reason: &str) -> Self {
        Self(format!("Invalid `{field}`: {reason}"))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Implemented by every request view extracted with [`ValidatedJson`] or [`ValidatedQuery`].
pub trait Validate {
    /// # Errors
    ///
    /// Returns the first field that does not satisfy its constraints.
    fn validate(&self) -> Result<(), ValidationError>;
}

fn check_length(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    if value.chars().count() > max {
        return Err(ValidationError::new(
            field,
            &format!("must be at most {max} characters"),
        ));
    }
    Ok(())
}

fn check_no_control(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.chars().any(char::is_control) {
        return Err(ValidationError::new(
            field,
            "must not contain control characters",
        ));
    }
    Ok(())
}

fn check_no_markup(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.contains(['<', '>']) {
        return Err(ValidationError::new(field, "must not contain `<` or `>`"));
    }
    Ok(())
}

/// A short label displayed as-is by the fronts (person name, role or group name): not blank, at
/// most `max` characters, no control character and no `<` / `>`.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the rules is broken.
pub fn check_label(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new(field, "must not be empty"));
    }
    check_length(field, value, max)?;
    check_no_control(field, value)?;
    check_no_markup(field, value)
}

/// A free-text description: may be empty, at most `max` characters, line breaks and tabs
/// allowed, no other control character and no `<` / `>`.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the rules is broken.
pub fn check_description(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    check_length(field, value, max)?;
    if value
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(ValidationError::new(
            field,
            "must not contain control characters other than line breaks and tabs",
        ));
    }
    check_no_markup(field, value)
}

/// An opaque value only compared or stored as text (token, credential, `device_info`, search
/// filter): at most `max` characters and no control character (Postgres rejects NUL bytes).
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the rules is broken.
pub fn check_opaque(field: &str, value: &str, max: usize) -> Result<(), ValidationError> {
    check_length(field, value, max)?;
    check_no_control(field, value)
}

/// A list of user ids: at most `max` entries, each between 1 and [`MAX_USER_ID`], no duplicate.
/// `allow_empty` tells whether an empty list is accepted.
///
/// # Errors
///
/// Returns a [`ValidationError`] naming `field` when one of the rules is broken.
pub fn check_user_ids(
    field: &str,
    ids: &[u64],
    max: usize,
    allow_empty: bool,
) -> Result<(), ValidationError> {
    if !allow_empty && ids.is_empty() {
        return Err(ValidationError::new(field, "must not be empty"));
    }
    if ids.len() > max {
        return Err(ValidationError::new(
            field,
            &format!("must hold at most {max} users"),
        ));
    }
    if ids.iter().any(|id| *id == 0 || *id > MAX_USER_ID) {
        return Err(ValidationError::new(
            field,
            &format!("ids must be between 1 and {MAX_USER_ID}"),
        ));
    }
    let mut sorted = ids.to_vec();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ValidationError::new(field, "must not contain duplicates"));
    }
    Ok(())
}

/// Runs `check` on `value` when it is present.
///
/// # Errors
///
/// Returns the error of `check`.
pub fn check_optional<F>(value: Option<&str>, check: F) -> Result<(), ValidationError>
where
    F: FnOnce(&str) -> Result<(), ValidationError>,
{
    value.map_or(Ok(()), check)
}

fn bad_request(error: &ValidationError) -> actix_web::Error {
    actix_web::error::ErrorBadRequest(error.to_string())
}

/// `web::Json<T>` followed by [`Validate::validate`]: answers `400` when either fails.
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let json = web::Json::<T>::from_request(req, payload);
        Box::pin(async move {
            let value = json.await?.into_inner();
            value.validate().map_err(|e| bad_request(&e))?;
            Ok(Self(value))
        })
    }
}

/// `web::Query<T>` followed by [`Validate::validate`]: answers `400` when either fails.
pub struct ValidatedQuery<T>(pub T);

impl<T> ValidatedQuery<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let query = web::Query::<T>::from_query(req.query_string());
        Box::pin(async move {
            let value = query?.into_inner();
            value.validate().map_err(|e| bad_request(&e))?;
            Ok(Self(value))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_rejects_blank_long_control_and_markup() {
        assert!(check_label("name", "Service urbanisme", MAX_TITLE_LENGTH).is_ok());
        assert!(check_label("name", "  ", MAX_TITLE_LENGTH).is_err());
        assert!(check_label("name", &"a".repeat(151), MAX_TITLE_LENGTH).is_err());
        assert!(check_label("name", "Service\0", MAX_TITLE_LENGTH).is_err());
        assert!(check_label("name", "<script>alert(1);</script>", MAX_TITLE_LENGTH).is_err());
    }

    #[test]
    fn label_counts_characters_not_bytes() {
        assert!(check_label("name", &"é".repeat(150), MAX_TITLE_LENGTH).is_ok());
    }

    #[test]
    fn description_allows_line_breaks_only() {
        assert!(check_description("content", "Bonjour,\nà demain", MAX_MESSAGE_LENGTH).is_ok());
        assert!(check_description("content", "a\0b", MAX_MESSAGE_LENGTH).is_err());
        assert!(check_description("content", "<b>", MAX_MESSAGE_LENGTH).is_err());
        assert!(check_description("content", &"a".repeat(5001), MAX_MESSAGE_LENGTH).is_err());
    }

    #[test]
    fn user_ids_are_bounded_in_range_and_unique() {
        assert!(check_user_ids("users_id", &[42, 51], 50, false).is_ok());
        assert!(check_user_ids("users_id", &[], 50, false).is_err());
        assert!(check_user_ids("members", &[], 50, true).is_ok());
        assert!(check_user_ids("users_id", &[0], 50, false).is_err());
        assert!(check_user_ids("users_id", &[MAX_USER_ID + 1], 50, false).is_err());
        assert!(check_user_ids("users_id", &[MAX_USER_ID], 50, false).is_ok());
        assert!(check_user_ids("users_id", &[42, 42], 50, false).is_err());
        let too_many: Vec<u64> = (1..=51).collect();
        assert!(check_user_ids("users_id", &too_many, 50, false).is_err());
    }

    #[test]
    fn optional_skips_absent_values() {
        assert!(check_optional(None, |v| check_label("name", v, MAX_TITLE_LENGTH)).is_ok());
        assert!(check_optional(Some(" "), |v| check_label("name", v, MAX_TITLE_LENGTH)).is_err());
    }
}
