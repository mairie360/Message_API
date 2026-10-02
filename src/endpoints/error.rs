//! Mapping of the database errors met by the handlers (MAIR-421).
//!
//! Every handler sends its `ApiLibError` through [`classify`]: the failures a client can cause
//! (row not found, foreign or unique key violation) come back as a [`DbFailure`] the handler maps
//! to its documented `4xx`; anything else is logged here, with the operation and the cause, and
//! becomes the handler's `500`. Nothing is swallowed without a log line.

use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;

/// What a database error means for the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbFailure {
    /// The statement matched no row (`RETURNING` / `fetch_one` / `fetch_scalar`).
    NotFound,
    /// A foreign key rejected a value of the request; holds the Postgres message (constraint name).
    ForeignKey(String),
    /// A unique key rejected a duplicate; holds the Postgres message (constraint name).
    Unique(String),
    /// Anything else (connection, SQL, mapping): already logged, answer `500`.
    Unexpected,
}

/// Classifies `error`, raised by `operation` (e.g. `"post message"`), and logs it when it is not
/// a client error.
pub fn classify(operation: &'static str, error: ApiLibError) -> DbFailure {
    match error {
        ApiLibError::Database(DbError::NotFound) => DbFailure::NotFound,
        ApiLibError::Database(DbError::ForeignKeyViolation(message)) => {
            tracing::debug!(operation, %message, "foreign key violation");
            DbFailure::ForeignKey(message)
        }
        ApiLibError::Database(DbError::UniqueViolation(message)) => {
            tracing::debug!(operation, %message, "unique violation");
            DbFailure::Unique(message)
        }
        error => {
            tracing::error!(operation, error = %error, "database error");
            DbFailure::Unexpected
        }
    }
}

/// Logs `error`, raised by `operation`, as the cause of a `500` regardless of its kind: for the
/// statements where even "not found" or a key violation is unexpected.
pub fn unexpected(operation: &'static str, error: ApiLibError) {
    tracing::error!(operation, error = %error, "database error");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_errors_are_told_apart_from_failures() {
        assert_eq!(
            classify("test", ApiLibError::Database(DbError::NotFound)),
            DbFailure::NotFound
        );
        assert_eq!(
            classify(
                "test",
                ApiLibError::Database(DbError::ForeignKeyViolation("fk_x".into()))
            ),
            DbFailure::ForeignKey("fk_x".into())
        );
        assert_eq!(
            classify(
                "test",
                ApiLibError::Database(DbError::UniqueViolation("uq_x".into()))
            ),
            DbFailure::Unique("uq_x".into())
        );
        assert_eq!(
            classify(
                "test",
                ApiLibError::Database(DbError::MappingError("bad".into()))
            ),
            DbFailure::Unexpected
        );
    }
}
