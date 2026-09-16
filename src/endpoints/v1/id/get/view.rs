use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use crate::database::chats::get_chat::view::Message;

/// Message d'une conversation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct MessageView {
    /// Identifiant du message.
    #[schema(example = 101)]
    id: u64,
    /// Contenu du message.
    #[schema(example = "La réunion est décalée à 15h.")]
    content: String,
    /// Identifiant Core API de l'auteur du message.
    #[schema(example = 42)]
    sender_id: u64,
    /// Date de publication.
    #[schema(value_type = String, format = DateTime, example = "2026-09-16T09:12:00Z")]
    created_at: DateTime<Utc>,
    /// Message auquel celui-ci répond. **Toujours `null` en lecture** : la valeur envoyée à la
    /// publication n'est pas encore relue depuis la base.
    sitation: Option<u64>, // message sitation
}

impl MessageView {
    pub fn new(
        id: u64,
        content: &str,
        sender_id: u64,
        created_at: DateTime<Utc>,
        sitation: Option<u64>,
    ) -> Self {
        Self {
            id,
            content: content.to_string(),
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

impl From<Message> for MessageView {
    fn from(message: Message) -> Self {
        Self::new(
            message.id as u64,
            &message.content,
            message.owner_id as u64,
            message.created_at,
            None,
        )
    }
}

/// Messages d'une conversation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, ToSchema)]
pub struct GetChatResultView {
    /// Tous les messages de la conversation, sans pagination. Vide si la conversation n'existe
    /// pas ou ne contient aucun message.
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

impl From<Vec<Message>> for GetChatResultView {
    fn from(messages: Vec<Message>) -> Self {
        Self::new(messages.into_iter().map(|m| m.into()).collect())
    }
}
