use utoipa::ToSchema;

use crate::database::chats::get_chats::view::GetChatsQueryResultView;

/// Conversation de l'utilisateur connecté, avec son nombre de messages non lus.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ChatView {
    /// Identifiant de la conversation, à réutiliser dans `/api/v1/{chat_id}/`.
    #[schema(example = 5)]
    id: u64,
    /// Titre de la conversation. Chaîne vide si elle n'en a pas — jamais `null`.
    #[schema(example = "Service urbanisme")]
    name: String,
    /// Messages non lus par l'utilisateur connecté. La consultation via
    /// `GET /api/v1/{chat_id}/` ne modifie pas ce compteur.
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
            result.id as u64,
            result.title.unwrap_or_default(),
            result.unread_count,
        )
    }
}

/// Conversations dont l'utilisateur connecté est participant.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetChatsResultView {
    /// Conversations dont l'utilisateur connecté est participant.
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
