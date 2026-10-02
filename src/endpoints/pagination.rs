//! Offset pagination of the lists (MAIR-425): `GET /api/v1/` and `GET /api/v1/{chat_id}/users/`.
//! The messages of a chat use a keyset instead (`before`, see `v1::id::get`).

use utoipa::IntoParams;

use crate::endpoints::validation::{Validate, ValidationError};

/// Items returned when `limit` is not given.
pub const DEFAULT_PAGE_SIZE: u32 = 50;
/// Largest `limit` accepted.
pub const MAX_PAGE_SIZE: u32 = 100;
/// Largest `offset` accepted (`INTEGER` in Postgres).
pub const MAX_OFFSET: u32 = i32::MAX.unsigned_abs();

/// Page of a list: `limit` items after skipping `offset`.
#[derive(Debug, Default, Clone, Copy, serde::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PageQuery {
    /// Number of items to return, 1 to 100. Default 50.
    #[param(example = 50, minimum = 1, maximum = 100)]
    limit: Option<u32>,
    /// Number of items to skip: pass the previous `offset` plus the previous `limit` while
    /// `has_more` is `true`. Default 0.
    #[param(example = 0, minimum = 0)]
    offset: Option<u32>,
}

impl PageQuery {
    pub fn new(limit: Option<u32>, offset: Option<u32>) -> Self {
        Self { limit, offset }
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(DEFAULT_PAGE_SIZE)
    }

    pub fn offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}

impl Validate for PageQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        if matches!(self.limit, Some(limit) if limit == 0 || limit > MAX_PAGE_SIZE) {
            return Err(ValidationError::new(
                "limit",
                &format!("must be between 1 and {MAX_PAGE_SIZE}"),
            ));
        }
        if matches!(self.offset, Some(offset) if offset > MAX_OFFSET) {
            return Err(ValidationError::new(
                "offset",
                &format!("must be between 0 and {MAX_OFFSET}"),
            ));
        }
        Ok(())
    }
}

/// Trims the `limit + 1` rows fetched for a page to `limit` and tells whether there are more.
pub fn split_page<T>(mut rows: Vec<T>, limit: u32) -> (Vec<T>, bool) {
    let limit = usize::try_from(limit).unwrap_or(usize::MAX);
    let has_more = rows.len() > limit;
    rows.truncate(limit);
    (rows, has_more)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_query_defaults_and_bounds() {
        let page = PageQuery::default();
        assert_eq!((page.limit(), page.offset()), (DEFAULT_PAGE_SIZE, 0));
        assert!(page.validate().is_ok());
        assert!(PageQuery::new(Some(0), None).validate().is_err());
        assert!(PageQuery::new(Some(101), None).validate().is_err());
        assert!(PageQuery::new(Some(100), Some(MAX_OFFSET))
            .validate()
            .is_ok());
        assert!(PageQuery::new(None, Some(MAX_OFFSET + 1))
            .validate()
            .is_err());
    }

    #[test]
    fn split_page_trims_the_lookahead_row() {
        assert_eq!(split_page(vec![1, 2, 3], 2), (vec![1, 2], true));
        assert_eq!(split_page(vec![1, 2], 2), (vec![1, 2], false));
        assert_eq!(split_page(Vec::<u8>::new(), 2), (vec![], false));
    }
}
