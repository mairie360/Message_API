use chrono::{DateTime, Utc};
use utoipa::{IntoParams, ToSchema};

use crate::database::chats::get_chat::view::{LinkRow, Message, ReactionRow, SystemEventRow};
use crate::database::ids::{bigint_from_sql, id_from_sql};
use crate::endpoints::v1::get::view::ChatView;
use crate::endpoints::validation::{Validate, ValidationError, MAX_MESSAGE_ID};

/// Same page size bounds as the other lists.
pub use crate::endpoints::pagination::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};

/// Pagination of the messages of a chat: the history (keyset on the message id, `before`), or the
/// changes since a cursor (`since`, `since_id`).
#[derive(Debug, Default, serde::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetChatQuery {
    /// History only: return messages older than this message id (exclusive), passing the
    /// `next_before` of the previous page. Absent = the latest messages. Not allowed with `since`.
    #[param(example = 118, minimum = 1)]
    before: Option<u64>,
    /// Number of messages to return, 1 to 100. Default 50.
    #[param(example = 50, minimum = 1, maximum = 100)]
    limit: Option<u32>,
    /// Incremental read (MAIR-524): only the messages changed after this cursor (posted, edited or
    /// deleted), in change order. RFC 3339 date-time, URL-encoded (`+` of an offset as `%2B`); pass
    /// the `next_since` of the previous response. Not allowed with `before`.
    #[param(value_type = Option<String>, format = DateTime, example = "2026-09-16T09:12:00.482113Z")]
    since: Option<DateTime<Utc>>,
    /// Second half of the cursor, with `since` only: the `next_since_id` of the previous response,
    /// so that the changes sharing the `since` instant are not returned twice nor skipped.
    /// Default 0: every change after `since` and the ones at exactly `since`.
    #[param(example = 118, minimum = 0)]
    since_id: Option<u64>,
}

impl GetChatQuery {
    pub fn new(before: Option<u64>, limit: Option<u32>) -> Self {
        Self {
            before,
            limit,
            since: None,
            since_id: None,
        }
    }

    /// The changes after the cursor `(since, since_id)`.
    pub fn since(since: DateTime<Utc>, since_id: Option<u64>, limit: Option<u32>) -> Self {
        Self {
            before: None,
            limit,
            since: Some(since),
            since_id,
        }
    }

    pub fn before(&self) -> Option<u64> {
        self.before
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(DEFAULT_PAGE_SIZE)
    }

    /// The cursor of an incremental read, `None` for the history.
    pub fn cursor(&self) -> Option<(DateTime<Utc>, u64)> {
        self.since
            .map(|since| (since, self.since_id.unwrap_or_default()))
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
        if self.since.is_some() && self.before.is_some() {
            return Err(ValidationError::new(
                "since",
                "cannot be combined with `before`: read the history or the changes",
            ));
        }
        if self.since_id.is_some() && self.since.is_none() {
            return Err(ValidationError::new("since_id", "requires `since`"));
        }
        if matches!(self.since_id, Some(id) if id > MAX_MESSAGE_ID) {
            return Err(ValidationError::new(
                "since_id",
                &format!("must be between 0 and {MAX_MESSAGE_ID}"),
            ));
        }
        Ok(())
    }
}

/// The message another one answers, with what is needed to display the quote without reading it.
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct QuotedMessageView {
    /// Id of the quoted message (the `citation` of the message that quotes it).
    #[schema(example = 117)]
    id: u64,
    /// Core API id of the author of the quoted message; `null` once the author's account is
    /// deleted.
    #[schema(example = 51, nullable = true)]
    sender_id: Option<u64>,
    /// First 100 characters of the quoted message.
    #[schema(example = "Any news about the building permit for rue Pasteur?")]
    excerpt: String,
}

impl QuotedMessageView {
    pub fn new(id: u64, sender_id: Option<u64>, excerpt: &str) -> Self {
        Self {
            id,
            sender_id,
            excerpt: excerpt.to_string(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn sender_id(&self) -> Option<u64> {
        self.sender_id
    }

    pub fn excerpt(&self) -> &str {
        &self.excerpt
    }
}

/// Who wrote a message (MAIR-1155).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum MessageKind {
    /// Written by a member: `content` holds the text.
    #[default]
    User,
    /// Written by the API when the members change: `content` is empty and `system_event` tells
    /// what happened. It never counts as unread, cannot be edited, deleted, quoted or reacted to.
    System,
}

/// What a system note records (MAIR-1155).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SystemEventAction {
    /// `actor_id` added `user_ids` to the chat.
    MembersAdded,
    /// `actor_id` removed `user_ids` (one member) from the chat.
    MemberRemoved,
    /// `actor_id` left the chat (`user_ids` holds `actor_id`).
    MemberLeft,
}

impl SystemEventAction {
    fn parse(action: &str) -> Option<Self> {
        match action {
            "members_added" => Some(Self::MembersAdded),
            "member_removed" => Some(Self::MemberRemoved),
            "member_left" => Some(Self::MemberLeft),
            _ => None,
        }
    }
}

/// The member change a system note records.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
pub struct SystemEventView {
    /// What happened.
    #[schema(example = "members_added")]
    action: SystemEventAction,
    /// Core API id of the member who made the change.
    #[schema(example = 42)]
    actor_id: u64,
    /// Core API ids of the members added, removed or leaving.
    #[schema(example = json!([51, 63]))]
    user_ids: Vec<u64>,
}

impl SystemEventView {
    pub fn new(action: SystemEventAction, actor_id: u64, user_ids: Vec<u64>) -> Self {
        Self {
            action,
            actor_id,
            user_ids,
        }
    }

    pub fn action(&self) -> SystemEventAction {
        self.action
    }

    pub fn actor_id(&self) -> u64 {
        self.actor_id
    }

    pub fn user_ids(&self) -> &[u64] {
        &self.user_ids
    }

    fn from_row(row: SystemEventRow) -> Option<Self> {
        Some(Self::new(
            SystemEventAction::parse(&row.action)?,
            id_from_sql(row.actor_id),
            row.user_ids.into_iter().map(id_from_sql).collect(),
        ))
    }
}

/// Kind of business object a message links to (MAIR-1155).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    /// A project of Project API.
    Project,
    /// A task of Project API.
    Task,
    /// An event of Calendar API.
    Event,
}

impl LinkKind {
    /// The `kind` stored in the tags document and read back from the database.
    pub fn as_str(self) -> &'static str {
        match self {
            LinkKind::Project => "project",
            LinkKind::Task => "task",
            LinkKind::Event => "event",
        }
    }

    fn parse(kind: &str) -> Option<Self> {
        match kind {
            "project" => Some(Self::Project),
            "task" => Some(Self::Task),
            "event" => Some(Self::Event),
            _ => None,
        }
    }
}

/// A business object a message links to: a project, a task or an event.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, ToSchema,
)]
pub struct MessageLinkView {
    /// Kind of the object.
    #[schema(example = "project")]
    kind: LinkKind,
    /// Id of the object in its API.
    #[schema(example = 12, minimum = 1, maximum = 2147483647)]
    id: u64,
    /// Answered only, for a task link: the project of the task (the Projects front opens a task at
    /// `?project=<project_id>&task=<id>`). Ignored in a request (MAIR-1166).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(example = 3, minimum = 1, maximum = 2147483647)]
    project_id: Option<u64>,
    /// Answered only, for an event link: the start of the event (the Calendar front opens an event
    /// at `?date=<its day>&event=<id>`). Ignored in a request (MAIR-1166).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-11-24T09:00:00Z")]
    start: Option<DateTime<Utc>>,
}

impl MessageLinkView {
    pub fn new(kind: LinkKind, id: u64) -> Self {
        Self {
            kind,
            id,
            project_id: None,
            start: None,
        }
    }

    /// The link as answered: with the project of a task and the start of an event.
    pub fn answered(
        kind: LinkKind,
        id: u64,
        project_id: Option<u64>,
        start: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            kind,
            id,
            project_id,
            start,
        }
    }

    pub fn project_id(&self) -> Option<u64> {
        self.project_id
    }

    pub fn start(&self) -> Option<DateTime<Utc>> {
        self.start
    }

    pub fn kind(&self) -> LinkKind {
        self.kind
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

/// The reactions of a message with one emoji.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
pub struct ReactionView {
    /// The emoji (1 to 32 characters, no whitespace).
    #[schema(example = "👍")]
    emoji: String,
    /// Number of members who reacted with it (`user_ids.len()`).
    #[schema(example = 2)]
    count: u32,
    /// Core API ids of the members who reacted with it, in reaction order.
    #[schema(example = json!([42, 51]))]
    user_ids: Vec<u64>,
}

impl ReactionView {
    pub fn emoji(&self) -> &str {
        &self.emoji
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn user_ids(&self) -> &[u64] {
        &self.user_ids
    }
}

impl From<ReactionRow> for ReactionView {
    fn from(row: ReactionRow) -> Self {
        Self {
            emoji: row.emoji,
            count: u32::try_from(row.count).unwrap_or_default(),
            user_ids: row.user_ids.into_iter().map(id_from_sql).collect(),
        }
    }
}

/// A message of a chat.
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct MessageView {
    /// Id of the message, usable as `before` and as `citation`.
    #[schema(example = 118)]
    id: u64,
    /// Content of the message.
    #[schema(example = "The meeting is moved to 3 pm.")]
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
    /// The message `citation` points to (author and excerpt), also when it is older than this page;
    /// `null` exactly when `citation` is.
    quoted: Option<QuotedMessageView>,
    /// Date of the last edit of the content; `null` when the message was never edited.
    #[schema(value_type = Option<String>, format = DateTime, example = "2026-09-16T09:14:30.105822Z", nullable = true, required = true)]
    edited_at: Option<DateTime<Utc>>,
    /// Date of the last change of the message (posted, edited or deleted), the order of the
    /// incremental reads (`since`).
    #[schema(value_type = String, format = DateTime, example = "2026-09-16T09:14:30.105822Z")]
    updated_at: DateTime<Utc>,
    /// `true` for a deleted message (tombstone), only returned by an incremental read (`since`):
    /// remove it from the display. Its `content` is then empty and `citation` / `quoted` are
    /// `null`. Always `false` in the history.
    #[schema(example = false)]
    deleted: bool,
    /// `user` for a message of a member, `system` for a note the API wrote when the members
    /// changed (MAIR-1155).
    #[schema(example = "user")]
    kind: MessageKind,
    /// `system` notes only: the member change they record; `null` for a message of a member.
    #[schema(nullable = true, required = true)]
    system_event: Option<SystemEventView>,
    /// Core API ids of the members the message mentions, by increasing id.
    #[schema(example = json!([51]))]
    mentions: Vec<u64>,
    /// Whether the message mentions every member of the chat (`@everyone`).
    #[schema(example = false)]
    mentions_everyone: bool,
    /// Core API ids of the groups the message mentions (their members present in the chat are
    /// notified), by increasing id.
    #[schema(example = json!([]))]
    group_mentions: Vec<u64>,
    /// Business objects the message links to, in the order they were given.
    links: Vec<MessageLinkView>,
    /// Reactions, one entry per emoji in the order of its first reaction; empty for a deleted
    /// message and a system note.
    reactions: Vec<ReactionView>,
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
            quoted: None,
            edited_at: None,
            updated_at: created_at,
            deleted: false,
            kind: MessageKind::User,
            system_event: None,
            mentions: Vec::new(),
            mentions_everyone: false,
            group_mentions: Vec::new(),
            links: Vec::new(),
            reactions: Vec::new(),
        }
    }

    /// Change tracking of the message (MAIR-524).
    pub fn with_changes(
        mut self,
        edited_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
        deleted: bool,
    ) -> Self {
        self.edited_at = edited_at;
        self.updated_at = updated_at;
        self.deleted = deleted;
        self
    }

    pub fn with_quoted(mut self, quoted: Option<QuotedMessageView>) -> Self {
        self.quoted = quoted;
        self
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

    pub fn quoted(&self) -> Option<&QuotedMessageView> {
        self.quoted.as_ref()
    }

    pub fn edited_at(&self) -> Option<&DateTime<Utc>> {
        self.edited_at.as_ref()
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn kind(&self) -> MessageKind {
        self.kind
    }

    pub fn system_event(&self) -> Option<&SystemEventView> {
        self.system_event.as_ref()
    }

    pub fn mentions(&self) -> &[u64] {
        &self.mentions
    }

    pub fn mentions_everyone(&self) -> bool {
        self.mentions_everyone
    }

    pub fn group_mentions(&self) -> &[u64] {
        &self.group_mentions
    }

    pub fn links(&self) -> &[MessageLinkView] {
        &self.links
    }

    pub fn reactions(&self) -> &[ReactionView] {
        &self.reactions
    }
}

impl From<Message> for MessageView {
    fn from(message: Message) -> Self {
        let quoted = match (
            message.reply_to_id.filter(|_| !message.deleted),
            message.reply_excerpt.as_deref(),
        ) {
            (Some(id), Some(excerpt)) => Some(QuotedMessageView::new(
                bigint_from_sql(id),
                message.reply_owner_id.map(id_from_sql),
                excerpt,
            )),
            _ => None,
        };
        let mut view = Self::new(
            bigint_from_sql(message.id),
            &message.content,
            message.owner_id.map(id_from_sql),
            message.created_at,
            message
                .reply_to_id
                .filter(|_| !message.deleted)
                .map(bigint_from_sql),
        )
        .with_quoted(quoted)
        .with_changes(message.edited_at, message.updated_at, message.deleted);
        view.system_event = message.system_event.and_then(SystemEventView::from_row);
        view.kind = if message.kind == "system" {
            MessageKind::System
        } else {
            MessageKind::User
        };
        view.mentions_everyone = message.mentions_everyone;
        view.mentions = message.mentions.into_iter().map(id_from_sql).collect();
        view.group_mentions = message
            .group_mentions
            .into_iter()
            .map(id_from_sql)
            .collect();
        view.links = message
            .links
            .into_iter()
            .filter_map(
                |LinkRow {
                     kind,
                     id,
                     project_id,
                     start,
                 }| {
                    LinkKind::parse(&kind).map(|kind| {
                        MessageLinkView::answered(
                            kind,
                            id_from_sql(id),
                            project_id.map(id_from_sql),
                            start,
                        )
                    })
                },
            )
            .collect();
        view.reactions = message
            .reactions
            .into_iter()
            .map(ReactionView::from)
            .collect();
        view
    }
}

/// One page of the messages of a chat: the history, or the changes since a cursor.
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct GetChatResultView {
    /// The chat itself, as a line of the chat list (`GET /api/v1/`): name to display, kind,
    /// contact, member count and the caller's unread counter. Read in the same call so that
    /// opening a chat needs nothing else but its members (`GET /api/v1/{chat_id}/users/`).
    chat: ChatView,
    /// History: messages of the page, **oldest first** (display order), never deleted ones; empty
    /// when the chat has no message older than `before`. Incremental read (`since`): the messages
    /// changed after the cursor in change order (`updated_at`, then `id`), tombstones included
    /// (`deleted: true`); empty when nothing changed.
    messages: Vec<MessageView>,
    /// Whether more remain: older messages (history, then pass `next_before` as `before`), or more
    /// changes (incremental read, then call again at once with `next_since` / `next_since_id`).
    #[schema(example = true)]
    has_more: bool,
    /// History: id of the oldest message of the page when `has_more` is `true`, `null` otherwise.
    /// Always `null` for an incremental read.
    #[schema(example = 101, nullable = true)]
    next_before: Option<u64>,
    /// Cursor to pass as `since` on the next incremental read, never `null`. Incremental read: the
    /// `updated_at` of the last message returned, or the `since` received when nothing changed.
    /// History: the `updated_at` of the latest change of the chat when this response was read (the
    /// creation of the chat when it has no message), so that polling from it misses nothing
    /// that happened after the page.
    #[schema(value_type = String, format = DateTime, example = "2026-09-16T09:12:00.482113Z")]
    next_since: DateTime<Utc>,
    /// Second half of the cursor, to pass as `since_id` with `next_since`: the id of that message
    /// (`0` when the chat has no message, or the `since_id` received).
    #[schema(example = 118, minimum = 0)]
    next_since_id: u64,
}

impl GetChatResultView {
    pub fn new(chat: ChatView, messages: Vec<MessageView>, has_more: bool) -> Self {
        let next_before = if has_more {
            messages.first().map(MessageView::id)
        } else {
            None
        };
        let next_since = chat.last_activity_at();
        Self {
            chat,
            messages,
            has_more,
            next_before,
            next_since,
            next_since_id: 0,
        }
    }

    /// The cursor to poll from after this response.
    pub fn with_cursor(mut self, since: DateTime<Utc>, since_id: u64) -> Self {
        self.next_since = since;
        self.next_since_id = since_id;
        self
    }

    /// Builds a page from the rows of `GetChatQueryView` run with `limit + 1` (newest first):
    /// the extra row only tells that older messages remain.
    pub fn from_newest_first(chat: ChatView, mut rows: Vec<Message>, limit: u32) -> Self {
        let has_more = rows.len() > limit as usize;
        rows.truncate(limit as usize);
        rows.reverse();
        Self::new(
            chat,
            rows.into_iter().map(MessageView::from).collect(),
            has_more,
        )
    }

    /// Builds an incremental read from the rows of `GetChatChangesQueryView` run with `limit + 1`
    /// after the cursor `(since, since_id)`: the extra row only tells that more changes remain,
    /// and the cursor moves to the last row returned (stays put when nothing changed).
    pub fn from_changes(
        chat: ChatView,
        mut rows: Vec<Message>,
        limit: u32,
        since: DateTime<Utc>,
        since_id: u64,
    ) -> Self {
        let has_more = rows.len() > limit as usize;
        rows.truncate(limit as usize);
        let (next_since, next_since_id) = rows.last().map_or((since, since_id), |row| {
            (row.updated_at, bigint_from_sql(row.id))
        });
        Self {
            chat,
            messages: rows.into_iter().map(MessageView::from).collect(),
            has_more,
            next_before: None,
            next_since,
            next_since_id,
        }
    }

    pub fn chat(&self) -> &ChatView {
        &self.chat
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

    pub fn next_since(&self) -> &DateTime<Utc> {
        &self.next_since
    }

    pub fn next_since_id(&self) -> u64 {
        self.next_since_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoints::v1::get::view::ChatKind;

    fn message(id: i64) -> Message {
        Message {
            id,
            owner_id: Some(42),
            content: format!("message {id}"),
            created_at: Utc::now(),
            reply_to_id: None,
            reply_owner_id: None,
            reply_excerpt: None,
            updated_at: Utc::now(),
            edited_at: None,
            deleted: false,
            kind: "user".to_string(),
            system_event: None,
            mentions_everyone: false,
            mentions: vec![],
            group_mentions: vec![],
            links: vec![],
            reactions: vec![],
        }
    }

    fn chat() -> ChatView {
        ChatView::new(
            5,
            "Urban planning department".to_string(),
            ChatKind::Group,
            None,
            8,
            0,
            Utc::now(),
        )
    }

    #[test]
    fn a_full_page_tells_where_to_continue() {
        let rows = vec![message(5), message(4), message(3)];
        let page = GetChatResultView::from_newest_first(chat(), rows, 2);
        let ids: Vec<u64> = page.messages().iter().map(MessageView::id).collect();
        assert_eq!(ids, vec![4, 5]);
        assert!(page.has_more());
        assert_eq!(page.next_before(), Some(4));
    }

    #[test]
    fn the_last_page_has_no_cursor() {
        let page = GetChatResultView::from_newest_first(chat(), vec![message(2), message(1)], 2);
        assert!(!page.has_more());
        assert_eq!(page.next_before(), None);
        assert_eq!(page.messages().len(), 2);
    }

    #[test]
    fn a_quote_comes_with_its_author_and_excerpt() {
        let mut quoting = message(7);
        quoting.reply_to_id = Some(3);
        quoting.reply_owner_id = Some(51);
        quoting.reply_excerpt = Some("Le permis ?".to_string());
        let view = MessageView::from(quoting);
        assert_eq!(view.citation(), Some(3));
        let quoted = view.quoted().expect("quoted message");
        assert_eq!(
            (quoted.id(), quoted.sender_id(), quoted.excerpt()),
            (3, Some(51), "Le permis ?")
        );
        assert!(MessageView::from(message(8)).quoted().is_none());
    }

    #[test]
    fn the_messaging_v2_fields_are_converted() {
        let mut note = message(9);
        note.kind = "system".to_string();
        note.content = String::new();
        note.system_event = Some(SystemEventRow {
            action: "members_added".to_string(),
            actor_id: 42,
            user_ids: vec![51, 63],
        });
        let view = MessageView::from(note);
        assert_eq!(view.kind(), MessageKind::System);
        let event = view.system_event().expect("system event");
        assert_eq!(
            (event.action(), event.actor_id(), event.user_ids()),
            (SystemEventAction::MembersAdded, 42, &[51, 63][..])
        );

        let mut tagged = message(10);
        tagged.mentions = vec![51];
        tagged.mentions_everyone = true;
        tagged.group_mentions = vec![3];
        tagged.links = vec![
            LinkRow {
                kind: "task".to_string(),
                id: 7,
                project_id: Some(3),
                start: None,
            },
            LinkRow {
                kind: "unknown".to_string(),
                id: 8,
                project_id: None,
                start: None,
            },
        ];
        tagged.reactions = vec![ReactionRow {
            emoji: "👍".to_string(),
            count: 2,
            user_ids: vec![42, 51],
        }];
        let view = MessageView::from(tagged);
        assert_eq!(view.kind(), MessageKind::User);
        assert!(view.system_event().is_none());
        assert_eq!(view.mentions(), &[51]);
        assert!(view.mentions_everyone());
        assert_eq!(view.group_mentions(), &[3]);
        assert_eq!(
            view.links(),
            &[MessageLinkView::answered(LinkKind::Task, 7, Some(3), None)]
        );
        let reaction = &view.reactions()[0];
        assert_eq!(
            (reaction.emoji(), reaction.count(), reaction.user_ids()),
            ("👍", 2, &[42, 51][..])
        );
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["kind"], "user");
        assert_eq!(
            json["links"][0],
            serde_json::json!({ "kind": "task", "id": 7, "project_id": 3 })
        );
        assert_eq!(json["system_event"], serde_json::Value::Null);
    }

    #[test]
    fn an_incremental_read_moves_the_cursor_to_its_last_row() {
        let since = Utc::now();
        let mut rows = vec![message(3), message(9), message(4)];
        rows[1].deleted = true;
        rows[1].content = String::new();
        rows[1].reply_to_id = Some(3);
        let page = GetChatResultView::from_changes(chat(), rows.clone(), 2, since, 0);
        let ids: Vec<u64> = page.messages().iter().map(MessageView::id).collect();
        assert_eq!(
            ids,
            vec![3, 9],
            "change order is kept, the extra row is dropped"
        );
        assert!(page.has_more());
        assert_eq!(page.next_before(), None);
        assert_eq!(page.next_since_id(), 9);
        assert_eq!(page.next_since(), &rows[1].updated_at);
        // A tombstone has no quote.
        assert!(page.messages()[1].deleted());
        assert_eq!(page.messages()[1].citation(), None);
    }

    #[test]
    fn an_empty_incremental_read_echoes_the_cursor() {
        let since = Utc::now();
        let page = GetChatResultView::from_changes(chat(), vec![], 50, since, 118);
        assert!(!page.has_more());
        assert_eq!((page.next_since(), page.next_since_id()), (&since, 118));
    }

    #[test]
    fn since_and_before_are_mutually_exclusive() {
        let since = GetChatQuery {
            before: Some(118),
            limit: None,
            since: Some(Utc::now()),
            since_id: None,
        };
        assert!(since
            .validate()
            .unwrap_err()
            .to_string()
            .starts_with("Invalid `since`"));
        let orphan = GetChatQuery {
            since_id: Some(3),
            ..GetChatQuery::default()
        };
        assert!(orphan
            .validate()
            .unwrap_err()
            .to_string()
            .starts_with("Invalid `since_id`"));
        assert!(GetChatQuery::since(Utc::now(), Some(0), Some(100))
            .validate()
            .is_ok());
        assert_eq!(GetChatQuery::new(None, None).cursor(), None);
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
