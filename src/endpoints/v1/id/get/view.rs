use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct MessageView {
    id: u64,
    content: String,
    sender_id: u64,
    #[schema(value_type = String, format = DateTime)]
    created_at: DateTime<Utc>,
    sitation: Option<u64>, // message sitation
}

impl MessageView {
    pub fn new(
        id: u64,
        content: String,
        sender_id: u64,
        created_at: DateTime<Utc>,
        sitation: Option<u64>,
    ) -> Self {
        Self {
            id,
            content,
            sender_id,
            created_at,
            sitation,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn sender_id(&self) -> u64 {
        self.sender_id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn sitation(&self) -> Option<u64> {
        self.sitation
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct GetChatResultView {
    messages: Vec<MessageView>,
}

impl GetChatResultView {
    pub fn new(messages: Vec<MessageView>) -> Self {
        Self { messages }
    }

    pub fn messages(&self) -> &[MessageView] {
        &self.messages
    }
}
