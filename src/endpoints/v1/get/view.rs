use utoipa::ToSchema;

use crate::database::chats::get_chats::view::GetChatsQueryResultView;
use crate::database::ids::id_from_sql;
use crate::endpoints::pagination::split_page;

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
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ChatView {
    /// Id of the chat, to use as `chat_id` in `/api/v1/{chat_id}/`.
    #[schema(example = 5)]
    id: u64,
    /// Title of the chat. Empty string for a chat without a title (every direct chat opened with
    /// `POST /api/v1/direct/`: show the contact's name instead), never `null`.
    #[schema(example = "Service urbanisme")]
    name: String,
    /// `direct` (two agents, see `contact_id`) or `group`.
    #[schema(example = "group")]
    kind: ChatKind,
    /// Core API id of the other participant of a direct chat, also when they hid it on their side.
    /// `null` for a group chat.
    #[schema(example = json!(null), nullable = true, required = true)]
    contact_id: Option<u64>,
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
        unread_count: i32,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            contact_id,
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
            result.unread_count,
        )
    }
}

/// One page of the chats of the caller, newest first.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetChatsResultView {
    /// Chats of the caller, newest first: at most `limit`. Empty when the caller is in no chat
    /// or `offset` is past the last one.
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
