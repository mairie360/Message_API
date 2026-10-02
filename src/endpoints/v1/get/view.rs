use utoipa::ToSchema;

use crate::database::chats::get_chats::view::GetChatsQueryResultView;
use crate::database::ids::id_from_sql;
use crate::endpoints::pagination::split_page;

/// A chat of the caller, with their number of unread messages.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ChatView {
    /// Id of the chat, to use as `chat_id` in `/api/v1/{chat_id}/`.
    #[schema(example = 5)]
    id: u64,
    /// Title of the chat. Empty string for a chat without a title, never `null`.
    #[schema(example = "Service urbanisme")]
    name: String,
    /// Messages the caller has not acknowledged yet. Reading `GET /api/v1/{chat_id}/` does not
    /// change it, only `POST /api/v1/{chat_id}/read/` does.
    #[schema(example = 3)]
    unread_count: i32,
}

impl ChatView {
    pub fn new(id: u64, name: String, unread_count: i32) -> Self {
        Self {
            id,
            name,
            unread_count,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
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
