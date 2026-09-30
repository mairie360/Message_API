use dashmap::DashMap;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;
use utoipa::ToSchema;

pub type SseSender = tokio::sync::mpsc::Sender<Result<actix_web::web::Bytes, String>>;

/// Identifies one SSE connection of a user (a user may have several open at once).
pub type ConnectionId = u64;

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

// L'instruction minimaliste que l'API REST va envoyer
#[derive(Clone, serde::Serialize)]
pub struct ChatEvent {
    pub chat_id: u64,
    pub sender_id: u64,
    pub message: String,
}

pub struct AppState {
    // SSE connections of each online agent (one entry per open tab/device)
    pub online_agents: DashMap<u64, Vec<(ConnectionId, SseSender)>>,
    // Le canal interne pour que le REST parle au SSE
    pub internal_bus: broadcast::Sender<ChatEvent>,
}

impl AppState {
    /// Registers a new SSE connection for `user_id` without touching its other connections.
    pub fn register_connection(&self, user_id: u64, sender: SseSender) -> ConnectionId {
        let id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);
        self.online_agents
            .entry(user_id)
            .or_default()
            .push((id, sender));
        id
    }

    /// Removes only the given connection; the user's entry disappears with its last connection.
    pub fn remove_connection(&self, user_id: u64, connection_id: ConnectionId) {
        self.online_agents
            .remove_if_mut(&user_id, |_, connections| {
                connections.retain(|(id, _)| *id != connection_id);
                connections.is_empty()
            });
    }

    /// Snapshot of the live connections of `user_id`.
    pub fn connections_of(&self, user_id: u64) -> Vec<(ConnectionId, SseSender)> {
        self.online_agents
            .get(&user_id)
            .map(|connections| connections.clone())
            .unwrap_or_default()
    }
}

/// Signal poussé sur le flux SSE quand une conversation change.
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatSignal {
    /// Le type d'événement (ex: "NEW_MSG"). Indique qu'il s'est passé quelque chose dans la
    /// conversation, sans porter le contenu : au client de recharger `GET /api/v1/{chat_id}/`.
    #[schema(example = "NEW_MSG")]
    pub r#type: String,

    /// L'identifiant du salon de discussion concerné, à recharger via `GET /api/v1/{chat_id}/`.
    #[schema(example = 5)]
    pub chat_id: u64,
}
