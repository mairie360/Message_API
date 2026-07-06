use utoipa::ToSchema;

use crate::database::chats::get_chats::view::GetChatsQueryResultView;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ChatView {
    id: u64,
    name: String,
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
            result.id as u64,
            result.title.unwrap_or_default(),
            result.unread_count,
        )
    }
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetChatsResultView {
    chats: Vec<ChatView>,
}

impl GetChatsResultView {
    pub fn new(chats: Vec<ChatView>) -> Self {
        Self { chats }
    }

    pub fn chats(&self) -> &[ChatView] {
        &self.chats
    }
}

impl From<Vec<GetChatsQueryResultView>> for GetChatsResultView {
    fn from(results: Vec<GetChatsQueryResultView>) -> Self {
        Self::new(results.into_iter().map(ChatView::from).collect())
    }
}
