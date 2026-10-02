use chrono::{DateTime, Utc};
use utoipa::{IntoParams, ToSchema};

use crate::database::chats::get_chat::view::Message;
use crate::database::ids::{bigint_from_sql, id_from_sql};
use crate::endpoints::validation::{Validate, ValidationError, MAX_MESSAGE_ID};

/// Same page size bounds as the other lists.
pub use crate::endpoints::pagination::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};

/// Pagination of the messages of a chat (keyset on the message id).
#[derive(Debug, Default, serde::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetChatQuery {
    /// Only return messages older than this message id (exclusive): pass the `next_before` of the
    /// previous page. Absent = the latest messages.
    #[param(example = 118, minimum = 1)]
    before: Option<u64>,
    /// Number of messages to return, 1 to 100. Default 50.
    #[param(example = 50, minimum = 1, maximum = 100)]
    limit: Option<u32>,
}

impl GetChatQuery {
    pub fn new(before: Option<u64>, limit: Option<u32>) -> Self {
        Self { before, limit }
    }

    pub fn before(&self) -> Option<u64> {
        self.before
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(DEFAULT_PAGE_SIZE)
    }
}

impl Validate for GetChatQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        if matches!(self.before, Some(id) if id == 0 || id > MAX_MESSAGE_ID) {
            return Err(ValidationError::new(
                "before",
                &format!("must be a message id between 1 and {MAX_MESSAGE_ID}"),
            ));
        }
        if matches!(self.limit, Some(limit) if limit == 0 || limit > MAX_PAGE_SIZE) {
            return Err(ValidationError::new(
                "limit",
                &format!("must be between 1 and {MAX_PAGE_SIZE}"),
            ));
        }
        Ok(())
    }
}

/// A message of a chat.
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct MessageView {
    /// Id of the message, usable as `before` and as `citation`.
    #[schema(example = 118)]
    id: u64,
    /// Content of the message.
    #[schema(example = "La réunion est décalée à 15h.")]
    content: String,
    /// Core API id of the author; `null` once the author's account is deleted.
    #[schema(example = 42, nullable = true)]
    sender_id: Option<u64>,
    /// Date the message was posted.
    #[schema(value_type = String, format = DateTime, example = "2026-09-16T09:12:00Z")]
    created_at: DateTime<Utc>,
    /// Id of the message this one answers (same chat); `null` when it answers none or once the
    /// quoted message is deleted.
    #[schema(example = 117, nullable = true)]
    citation: Option<u64>,
}

impl MessageView {
    pub fn new(
        id: u64,
        content: &str,
        sender_id: Option<u64>,
        created_at: DateTime<Utc>,
        citation: Option<u64>,
    ) -> Self {
        Self {
            id,
            content: content.to_string(),
            sender_id,
            created_at,
            citation,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn sender_id(&self) -> Option<u64> {
        self.sender_id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn citation(&self) -> Option<u64> {
        self.citation
    }
}

impl From<Message> for MessageView {
    fn from(message: Message) -> Self {
        Self::new(
            bigint_from_sql(message.id),
            &message.content,
            message.owner_id.map(id_from_sql),
            message.created_at,
            message.reply_to_id.map(bigint_from_sql),
        )
    }
}

/// One page of the messages of a chat.
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct GetChatResultView {
    /// Messages of the page, **oldest first** (display order). Empty when the chat has no
    /// message older than `before`.
    messages: Vec<MessageView>,
    /// Whether older messages remain: then pass `next_before` as `before` to get them.
    #[schema(example = true)]
    has_more: bool,
    /// Id of the oldest message of the page when `has_more` is `true`, `null` otherwise.
    #[schema(example = 101, nullable = true)]
    next_before: Option<u64>,
}

impl GetChatResultView {
    pub fn new(messages: Vec<MessageView>, has_more: bool) -> Self {
        let next_before = if has_more {
            messages.first().map(MessageView::id)
        } else {
            None
        };
        Self {
            messages,
            has_more,
            next_before,
        }
    }

    /// Builds a page from the rows of `GetChatQueryView` run with `limit + 1` (newest first):
    /// the extra row only tells that older messages remain.
    pub fn from_newest_first(mut rows: Vec<Message>, limit: u32) -> Self {
        let has_more = rows.len() > limit as usize;
        rows.truncate(limit as usize);
        rows.reverse();
        Self::new(rows.into_iter().map(MessageView::from).collect(), has_more)
    }

    pub fn messages(&self) -> &[MessageView] {
        &self.messages
    }

    pub fn has_more(&self) -> bool {
        self.has_more
    }

    pub fn next_before(&self) -> Option<u64> {
        self.next_before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(id: i64) -> Message {
        Message {
            id,
            owner_id: Some(42),
            content: format!("message {id}"),
            created_at: Utc::now(),
            reply_to_id: None,
        }
    }

    #[test]
    fn a_full_page_tells_where_to_continue() {
        let rows = vec![message(5), message(4), message(3)];
        let page = GetChatResultView::from_newest_first(rows, 2);
        let ids: Vec<u64> = page.messages().iter().map(MessageView::id).collect();
        assert_eq!(ids, vec![4, 5]);
        assert!(page.has_more());
        assert_eq!(page.next_before(), Some(4));
    }

    #[test]
    fn the_last_page_has_no_cursor() {
        let page = GetChatResultView::from_newest_first(vec![message(2), message(1)], 2);
        assert!(!page.has_more());
        assert_eq!(page.next_before(), None);
        assert_eq!(page.messages().len(), 2);
    }

    #[test]
    fn query_bounds() {
        assert!(GetChatQuery::new(None, None).validate().is_ok());
        assert_eq!(GetChatQuery::new(None, None).limit(), DEFAULT_PAGE_SIZE);
        assert!(GetChatQuery::new(Some(0), None).validate().is_err());
        assert!(GetChatQuery::new(None, Some(0)).validate().is_err());
        assert!(GetChatQuery::new(None, Some(101)).validate().is_err());
        assert!(GetChatQuery::new(Some(118), Some(100)).validate().is_ok());
    }
}
