use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ChatView {
    id: u64,
    name: String,
}

impl ChatView {
    pub fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
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
