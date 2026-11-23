use chrono::{DateTime, Utc};
use utoipa::{IntoParams, ToSchema};

use crate::database::chats::search::view::{MessageMatch, PersonMatch};
use crate::database::ids::{bigint_from_sql, id_from_sql};
use crate::endpoints::pagination::MAX_OFFSET;
use crate::endpoints::validation::{check_label, Validate, ValidationError, MAX_TITLE_LENGTH};

/// Items returned when `limit` is not given.
pub const DEFAULT_SEARCH_SIZE: u32 = 20;
/// Largest `limit` accepted.
pub const MAX_SEARCH_SIZE: u32 = 50;

/// What to search in a chat.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    /// The messages of the members (not the deleted ones nor the system notes), newest first.
    #[default]
    Messages,
    /// The members, by last name then first name.
    People,
    /// The files of the chat: always empty, there are no attachments yet.
    Files,
}

/// The search of `GET /api/v1/{chat_id}/search/`.
#[derive(Debug, serde::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchQuery {
    /// Text to find, case-insensitive, anywhere in the content (messages) or in the first name,
    /// last name or full name (people): 1 to 150 characters, not blank, no control character.
    #[param(example = "permis", min_length = 1, max_length = 150)]
    q: String,
    /// What to search: `messages` (default), `people` or `files`.
    #[param(inline, example = "messages")]
    r#type: Option<SearchType>,
    /// Number of items to return, 1 to 50. Default 20.
    #[param(example = 20, minimum = 1, maximum = 50)]
    limit: Option<u32>,
    /// Number of items to skip: the previous `offset` plus `limit` while `has_more`. Default 0.
    #[param(example = 0, minimum = 0)]
    offset: Option<u32>,
}

impl SearchQuery {
    pub fn new(
        q: &str,
        r#type: Option<SearchType>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Self {
        Self {
            q: q.to_string(),
            r#type,
            limit,
            offset,
        }
    }

    pub fn q(&self) -> &str {
        &self.q
    }

    pub fn search_type(&self) -> SearchType {
        self.r#type.unwrap_or_default()
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(DEFAULT_SEARCH_SIZE)
    }

    pub fn offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}

impl Validate for SearchQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        check_label("q", &self.q, MAX_TITLE_LENGTH)?;
        if matches!(self.limit, Some(limit) if limit == 0 || limit > MAX_SEARCH_SIZE) {
            return Err(ValidationError::new(
                "limit",
                &format!("must be between 1 and {MAX_SEARCH_SIZE}"),
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

/// A message matching the search.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
pub struct SearchMessageItem {
    /// Id of the message (open the chat with `before=<id + 1>` to show it).
    #[schema(example = 117)]
    id: u64,
    /// Core API id of the author; `null` once the author's account is deleted.
    #[schema(example = 51, nullable = true)]
    sender_id: Option<u64>,
    /// Date the message was posted.
    #[schema(value_type = String, format = DateTime, example = "2026-09-16T09:05:00Z")]
    created_at: DateTime<Utc>,
    /// Content of the message.
    #[schema(example = "Any news about the building permit for rue Pasteur?")]
    content: String,
}

impl SearchMessageItem {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl From<MessageMatch> for SearchMessageItem {
    fn from(row: MessageMatch) -> Self {
        Self {
            id: bigint_from_sql(row.id),
            sender_id: row.sender_id.map(id_from_sql),
            created_at: row.created_at,
            content: row.content,
        }
    }
}

/// A member matching the search.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
pub struct SearchPersonItem {
    /// Core API id of the member.
    #[schema(example = 42)]
    id: u64,
    #[schema(example = "Xavier")]
    first_name: String,
    #[schema(example = "Bertrand")]
    last_name: String,
}

impl SearchPersonItem {
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl From<PersonMatch> for SearchPersonItem {
    fn from(row: PersonMatch) -> Self {
        Self {
            id: id_from_sql(row.id),
            first_name: row.first_name,
            last_name: row.last_name,
        }
    }
}

/// One item of a search: a message (`type=messages`) or a member (`type=people`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
#[serde(untagged)]
pub enum SearchItem {
    Message(SearchMessageItem),
    Person(SearchPersonItem),
}

/// One page of the matches of a search in a chat.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
pub struct SearchResultView {
    /// What was searched (the `type` of the request, `messages` by default).
    #[schema(example = "messages")]
    r#type: SearchType,
    /// The matches of the page: messages newest first, or members by last name then first name.
    items: Vec<SearchItem>,
    /// `true` when more matches follow: call again with `offset` increased by `limit`.
    #[schema(example = false)]
    has_more: bool,
    /// Number of matches in all.
    #[schema(example = 1)]
    total: u32,
}

impl SearchResultView {
    pub fn new(r#type: SearchType, items: Vec<SearchItem>, offset: u32, total: u32) -> Self {
        let shown = u32::try_from(items.len()).unwrap_or(u32::MAX);
        Self {
            r#type,
            has_more: offset.saturating_add(shown) < total,
            items,
            total,
        }
    }

    pub fn items(&self) -> &[SearchItem] {
        &self.items
    }

    pub fn has_more(&self) -> bool {
        self.has_more
    }

    pub fn total(&self) -> u32 {
        self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds() {
        assert!(SearchQuery::new("permis", None, None, None)
            .validate()
            .is_ok());
        assert!(SearchQuery::new(" ", None, None, None).validate().is_err());
        assert!(SearchQuery::new(&"a".repeat(151), None, None, None)
            .validate()
            .is_err());
        assert!(SearchQuery::new("a\u{7}", None, None, None)
            .validate()
            .is_err());
        assert!(SearchQuery::new("a", None, Some(0), None)
            .validate()
            .is_err());
        assert!(SearchQuery::new("a", None, Some(51), None)
            .validate()
            .is_err());
        assert!(SearchQuery::new("a", None, None, Some(MAX_OFFSET + 1))
            .validate()
            .is_err());
        let query = SearchQuery::new("a", None, None, None);
        assert_eq!(
            (query.search_type(), query.limit(), query.offset()),
            (SearchType::Messages, DEFAULT_SEARCH_SIZE, 0)
        );
    }

    #[test]
    fn has_more_compares_the_page_with_the_total() {
        let item = SearchItem::Person(SearchPersonItem {
            id: 1,
            first_name: "A".into(),
            last_name: "B".into(),
        });
        assert!(SearchResultView::new(SearchType::People, vec![item.clone()], 0, 2).has_more());
        assert!(!SearchResultView::new(SearchType::People, vec![item], 1, 2).has_more());
        let empty = SearchResultView::new(SearchType::Files, vec![], 0, 0);
        assert_eq!(
            serde_json::to_value(&empty).unwrap(),
            serde_json::json!({ "type": "files", "items": [], "has_more": false, "total": 0 })
        );
    }
}
