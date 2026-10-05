//! Conversions between the API's `u64` ids and the Postgres columns (MAIR-422).
//!
//! A plain `as` cast wraps around: `4294967301 as i32` is `5`, so `/api/v1/4294967301/` used to
//! answer for chat 5. These conversions saturate instead: an id beyond the column's range becomes
//! its maximum, which no row has, so the request ends in the same `404` as any unknown id.

pub use mairie360_api_lib::database::db_interface::{id_from_sql, id_to_sql};

/// `u64` id → `BIGINT` (`messages.id`), saturating at `i64::MAX`.
pub fn bigint_to_sql(id: u64) -> i64 {
    i64::try_from(id).unwrap_or(i64::MAX)
}

/// `BIGINT` → `u64` id; a negative value (never produced by a sequence) gives `0`.
pub fn bigint_from_sql(id: i64) -> u64 {
    u64::try_from(id).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_range_ids_saturate_instead_of_wrapping() {
        assert_eq!(id_to_sql(5), 5);
        assert_eq!(id_to_sql((1 << 32) + 5), i32::MAX);
        assert_eq!(id_to_sql(u64::MAX), i32::MAX);
        assert_eq!(bigint_to_sql(118), 118);
        assert_eq!(bigint_to_sql(u64::MAX), i64::MAX);
        assert_eq!(id_from_sql(-1), 0);
        assert_eq!(bigint_from_sql(-1), 0);
        assert_eq!(bigint_from_sql(i64::MAX), i64::MAX as u64);
    }
}
