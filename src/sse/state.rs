use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Notify};
use utoipa::ToSchema;

use crate::sse::relay::RedisRelay;

pub type SseSender = tokio::sync::mpsc::Sender<Result<actix_web::web::Bytes, String>>;

/// Identifies one SSE connection of a user (a user may have several open at once).
pub type ConnectionId = u64;

/// Open streams kept per user (tabs, devices). Opening one more closes the oldest.
pub const MAX_CONNECTIONS_PER_USER: usize = 5;

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// What a write endpoint announces on the bus: something happened in `chat_id`. It carries no
/// content, the clients reload the chat. Serialized as is on the Redis channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatEvent {
    pub chat_id: u64,
    pub sender_id: u64,
}

/// One open SSE stream. `close` ends the HTTP response when it is notified (eviction, expired or
/// revoked token).
#[derive(Clone)]
pub struct Connection {
    pub id: ConnectionId,
    pub sender: SseSender,
    pub close: Arc<Notify>,
}

pub struct AppState {
    /// SSE connections of each online agent (one entry per open tab/device), oldest first.
    pub online_agents: DashMap<u64, Vec<Connection>>,
    /// Events to fan out to the connections of this process (see `event_manager`).
    pub internal_bus: broadcast::Sender<ChatEvent>,
    /// Redis pub/sub relay sharing the events between replicas; `None` = this process only.
    pub relay: Option<Arc<RedisRelay>>,
}

impl AppState {
    pub fn new(internal_bus: broadcast::Sender<ChatEvent>, relay: Option<Arc<RedisRelay>>) -> Self {
        Self {
            online_agents: DashMap::new(),
            internal_bus,
            relay,
        }
    }

    /// Announces `event` to every replica. Through Redis when the relay is up (each replica,
    /// this one included, gets it back from its subscription); straight on the local bus
    /// otherwise, so the members connected to this replica are still notified. A signal may be
    /// delivered twice while the relay reconnects: it only makes a client reload once more.
    pub async fn publish(&self, event: ChatEvent) {
        let relayed = match &self.relay {
            Some(relay) => relay.publish(&event).await && relay.is_subscribed(),
            None => false,
        };
        if !relayed {
            // No receiver (listener not started yet) is not an error worth reporting.
            let _ = self.internal_bus.send(event);
        }
    }

    /// Registers a new SSE connection for `user_id` without touching its other connections.
    pub fn register_connection(&self, user_id: u64, sender: SseSender) -> ConnectionId {
        self.register(user_id, sender).id
    }

    /// Registers a new SSE connection for `user_id`. Beyond [`MAX_CONNECTIONS_PER_USER`] the
    /// oldest connections of that user are removed and told to close.
    pub fn register(&self, user_id: u64, sender: SseSender) -> Connection {
        let connection = Connection {
            id: NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
            sender,
            close: Arc::new(Notify::new()),
        };
        let evicted: Vec<Connection> = {
            let mut connections = self.online_agents.entry(user_id).or_default();
            connections.push(connection.clone());
            let excess = connections.len().saturating_sub(MAX_CONNECTIONS_PER_USER);
            connections.drain(..excess).collect()
        };
        for old in evicted {
            // `notify_one` keeps a permit: the stream ends even if it is not polled yet.
            old.close.notify_one();
        }
        connection
    }

    /// Removes only the given connection; the user's entry disappears with its last connection.
    pub fn remove_connection(&self, user_id: u64, connection_id: ConnectionId) {
        self.online_agents
            .remove_if_mut(&user_id, |_, connections| {
                connections.retain(|connection| connection.id != connection_id);
                connections.is_empty()
            });
    }

    /// Snapshot of the live connections of `user_id`.
    pub fn connections_of(&self, user_id: u64) -> Vec<(ConnectionId, SseSender)> {
        self.online_agents
            .get(&user_id)
            .map(|connections| {
                connections
                    .iter()
                    .map(|connection| (connection.id, connection.sender.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Signal pushed on the SSE stream when a chat changes.
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatSignal {
    /// Event type (`NEW_MSG`). Says that something happened in the chat, without the content:
    /// the client reloads `GET /api/v1/{chat_id}/`.
    #[schema(example = "NEW_MSG")]
    pub r#type: String,

    /// Id of the chat concerned, to reload with `GET /api/v1/{chat_id}/`.
    #[schema(example = 5)]
    pub chat_id: u64,
}
