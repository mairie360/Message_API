use utoipa::{IntoParams, ToSchema};

use crate::database::chats::get_chats::view::GetChatsQueryResultView;
use crate::database::ids::id_from_sql;
use crate::endpoints::pagination::{split_page, PageQuery};
use crate::endpoints::validation::{Validate, ValidationError, MAX_TITLE_LENGTH};

/// Kind of a chat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChatKind {
    /// Conversation between exactly two agents, opened with `POST /api/v1/direct/`: one per pair,
    /// `contact_id` is the other participant. Hiding it (`DELETE /api/v1/{chat_id}/users/{own id}/`)
    /// only removes it from the list until the next message.
    Direct,
    /// Any other chat, created with `POST /api/v1/`: titled, with members added and removed by its
    /// creator or an administrator.
    Group,
}

impl ChatKind {
    /// `conversations.kind` → kind; anything but `direct` is a group chat.
    pub fn from_sql(kind: &str) -> Self {
        if kind == "direct" {
            ChatKind::Direct
        } else {
            ChatKind::Group
        }
    }
}

/// A chat of the caller, with their number of unread messages.
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct ChatView {
    /// Id of the chat, to use as `chat_id` in `/api/v1/{chat_id}/`.
    #[schema(example = 5)]
    id: u64,
    /// Name to display. A group chat: its title (empty string when it has none). A direct chat: the
    /// full name of the other participant ("Prénom Nom"), computed by the API. Never `null`.
    #[schema(example = "Service urbanisme")]
    name: String,
    /// `direct` (two agents, see `contact_id`) or `group`.
    #[schema(example = "group")]
    kind: ChatKind,
    /// Core API id of the other participant of a direct chat, also when they hid it on their side.
    /// `null` for a group chat.
    #[schema(example = json!(null), nullable = true, required = true)]
    contact_id: Option<u64>,
    /// Number of members of a group chat (those who did not leave it); always 2 for a direct chat,
    /// whose participants never change. The ids and names of the members are read with
    /// `GET /api/v1/{chat_id}/users/`.
    #[schema(example = 8)]
    member_count: u32,
    /// Messages the caller has not acknowledged yet. Reading `GET /api/v1/{chat_id}/` does not
    /// change it, only `POST /api/v1/{chat_id}/read/` does.
    #[schema(example = 3)]
    unread_count: i32,
}

impl ChatView {
    pub fn new(
        id: u64,
        name: String,
        kind: ChatKind,
        contact_id: Option<u64>,
        member_count: u32,
        unread_count: i32,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            contact_id,
            member_count,
            unread_count,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> ChatKind {
        self.kind
    }

    pub fn contact_id(&self) -> Option<u64> {
        self.contact_id
    }

    pub fn member_count(&self) -> u32 {
        self.member_count
    }

    pub fn unread_count(&self) -> i32 {
        self.unread_count
    }
}

impl From<GetChatsQueryResultView> for ChatView {
    fn from(result: GetChatsQueryResultView) -> Self {
        Self::new(
            id_from_sql(result.id),
            result.title.unwrap_or_default(),
            ChatKind::from_sql(&result.kind),
            result.contact_id.map(id_from_sql),
            u32::try_from(result.member_count).unwrap_or_default(),
            result.unread_count,
        )
    }
}

/// Longest `search` accepted: the longest name of a chat (`conversations.title` is `VARCHAR(150)`).
pub const MAX_SEARCH_LENGTH: usize = MAX_TITLE_LENGTH;

/// Page and search of `GET /api/v1/`.
#[derive(Debug, Default, Clone, serde::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ChatListQuery {
    /// Number of chats to return, 1 to 100. Default 50.
    #[param(example = 50, minimum = 1, maximum = 100)]
    limit: Option<u32>,
    /// Number of chats to skip: pass the previous `offset` plus the previous `limit` while
    /// `has_more` is `true`. Default 0.
    #[param(example = 0, minimum = 0)]
    offset: Option<u32>,
    /// Keeps the chats whose name contains this text, or in which a member other than the caller
    /// has a first name, a last name or a full name ("Prénom Nom" or "Nom Prénom") containing it:
    /// "autoroute" finds the chat "Projet autoroute", "Xavier Bertrand" finds the direct chat with
    /// him and the group chats he belongs to. Case-insensitive (accents count); surrounding
    /// spaces are ignored; at most 150 characters, no control character. The page is taken from
    /// the filtered list.
    #[param(example = "Xavier Bertrand", max_length = 150)]
    search: Option<String>,
}

impl ChatListQuery {
    pub fn new(limit: Option<u32>, offset: Option<u32>, search: Option<String>) -> Self {
        Self {
            limit,
            offset,
            search,
        }
    }

    pub fn page(&self) -> PageQuery {
        PageQuery::new(self.limit, self.offset)
    }

    pub fn search(&self) -> Option<&str> {
        self.search.as_deref()
    }
}

impl Validate for ChatListQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        self.page().validate()?;
        if let Some(search) = &self.search {
            if search.chars().count() > MAX_SEARCH_LENGTH {
                return Err(ValidationError::new(
                    "search",
                    &format!("must be at most {MAX_SEARCH_LENGTH} characters"),
                ));
            }
            if search.chars().any(char::is_control) {
                return Err(ValidationError::new(
                    "search",
                    "must not contain control characters",
                ));
            }
        }
        Ok(())
    }
}

/// One page of the chats of the caller, newest first.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetChatsResultView {
    /// Chats of the caller (matching `search` when given), newest first: at most `limit`. Empty
    /// when there is none or `offset` is past the last one.
    chats: Vec<ChatView>,
    /// `true` when more chats follow: call again with `offset` increased by `limit`.
    #[schema(example = false)]
    has_more: bool,
}

impl GetChatsResultView {
    pub fn new(chats: Vec<ChatView>, has_more: bool) -> Self {
        Self { chats, has_more }
    }

    /// Builds the page from the `limit + 1` rows fetched (see `pagination::split_page`).
    pub fn from_rows(rows: Vec<GetChatsQueryResultView>, limit: u32) -> Self {
        let (rows, has_more) = split_page(rows, limit);
        Self::new(rows.into_iter().map(ChatView::from).collect(), has_more)
    }

    pub fn chats(&self) -> &[ChatView] {
        &self.chats
    }

    pub fn has_more(&self) -> bool {
        self.has_more
    }
}
