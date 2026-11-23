use utoipa::ToSchema;

/// Unread messages of the caller over all their chats.
#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct UnreadTotalView {
    /// Sum of the `unread_count` of the chats of `GET /api/v1/` (hidden direct chats are not counted).
    #[schema(example = 7)]
    pub unread_count: u32,
    /// Number of those chats with at least one unread message.
    #[schema(example = 3)]
    pub unread_chats: u32,
}

impl UnreadTotalView {
    pub fn new(unread_count: u32, unread_chats: u32) -> Self {
        Self {
            unread_count,
            unread_chats,
        }
    }
}
