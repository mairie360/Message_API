use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Notify};
use utoipa::ToSchema;

use crate::sse::relay::RedisRelay;

pub type SseSender = tokio::sync::mpsc::Sender<Result<actix_web::web::Bytes, String>>;

/// Identifies one real-time connection of a user (a user may have several open at once).
pub type ConnectionId = u64;

/// Open connections kept per user (tabs, devices), SSE and WebSocket counted together (MAIR-656).
/// Opening one more closes the oldest.
pub const MAX_CONNECTIONS_PER_USER: usize = 5;

/// How a connection receives its signals: the payload pushed on its channel differs (MAIR-657).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Transport {
    /// `GET /api/v1/stream`: each signal is a whole `data: {...}\n\n` SSE event.
    #[default]
    Sse,
    /// `GET /api/v1/ws`: each signal is the bare `ChatSignal` JSON, sent as one text frame.
    WebSocket,
}

impl Transport {
    /// What the connection's channel receives for the serialized `ChatSignal` `payload`.
    pub fn frame(self, payload: &str) -> actix_web::web::Bytes {
        match self {
            Transport::Sse => actix_web::web::Bytes::from(format!("data: {payload}\n\n")),
            Transport::WebSocket => actix_web::web::Bytes::copy_from_slice(payload.as_bytes()),
        }
    }
}

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// What happened in a chat (MAIR-524). Serialized in `ChatEvent` on the Redis channel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatEventKind {
    /// A message was posted (`NEW_MSG`). The default, so that an event published by a replica
    /// older than MAIR-524 (no `kind`) is still read as a new message.
    #[default]
    MessageCreated,
    /// The content of a message was edited (`MSG_EDITED`).
    MessageEdited,
    /// A message was deleted, it is now a tombstone (`MSG_DELETED`).
    MessageDeleted,
    /// The actor acknowledged the chat up to `message_id` (`CHAT_READ`), only signalled to the
    /// actor's own streams.
    ChatRead,
}

impl ChatEventKind {
    /// `type` of the `ChatSignal` frame pushed to the clients.
    pub fn signal_type(self) -> &'static str {
        match self {
            ChatEventKind::MessageCreated => "NEW_MSG",
            ChatEventKind::MessageEdited => "MSG_EDITED",
            ChatEventKind::MessageDeleted => "MSG_DELETED",
            ChatEventKind::ChatRead => "CHAT_READ",
        }
    }
}

/// What a write endpoint announces on the bus once its transaction is committed: something
/// happened in `chat_id`. It carries no content, the clients read the change with
/// `GET /api/v1/{chat_id}/?since=`. Serialized as is on the Redis channel; the fields added by
/// MAIR-524 default when absent and are ignored by an older replica, so a rolling update keeps
/// working (an older replica signals any event as `NEW_MSG`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatEvent {
    pub chat_id: u64,
    /// Author of the change: the sender, the editor, the deleting user or the reader. Never
    /// signalled, except for `ChatRead`, which only the reader's own streams receive.
    pub sender_id: u64,
    #[serde(default)]
    pub kind: ChatEventKind,
    /// The message created, edited or deleted, or the last message acknowledged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u64>,
    /// `ChatRead` only: messages still unread by the reader after the acknowledgement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unread_count: Option<i32>,
}

impl ChatEvent {
    fn of(kind: ChatEventKind, chat_id: u64, sender_id: u64, message_id: Option<u64>) -> Self {
        Self {
            chat_id,
            sender_id,
            kind,
            message_id,
            unread_count: None,
        }
    }

    /// `sender_id` posted `message_id` in `chat_id`.
    pub fn message_created(chat_id: u64, sender_id: u64, message_id: u64) -> Self {
        Self::of(
            ChatEventKind::MessageCreated,
            chat_id,
            sender_id,
            Some(message_id),
        )
    }

    /// `editor_id` edited `message_id` of `chat_id`.
    pub fn message_edited(chat_id: u64, editor_id: u64, message_id: u64) -> Self {
        Self::of(
            ChatEventKind::MessageEdited,
            chat_id,
            editor_id,
            Some(message_id),
        )
    }

    /// `deleted_by` deleted `message_id` of `chat_id` (its author, or an administrator).
    pub fn message_deleted(chat_id: u64, deleted_by: u64, message_id: u64) -> Self {
        Self::of(
            ChatEventKind::MessageDeleted,
            chat_id,
            deleted_by,
            Some(message_id),
        )
    }

    /// `reader_id` acknowledged `chat_id` up to `message_id`, `unread_count` messages remain.
    pub fn chat_read(chat_id: u64, reader_id: u64, message_id: u64, unread_count: i32) -> Self {
        Self {
            unread_count: Some(unread_count),
            ..Self::of(
                ChatEventKind::ChatRead,
                chat_id,
                reader_id,
                Some(message_id),
            )
        }
    }
}

/// One open real-time connection. `close` ends it when it is notified: the SSE response ends,
/// the WebSocket is closed with `4008` (eviction). Expired or revoked tokens are handled by each
/// transport's own supervisor.
#[derive(Clone)]
pub struct Connection {
    pub id: ConnectionId,
    pub sender: SseSender,
    pub close: Arc<Notify>,
    pub transport: Transport,
}

pub struct AppState {
    /// Real-time connections (SSE and WebSocket) of each online agent (one entry per open
    /// tab/device), oldest first.
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
    /// delivered twice while the relay reconnects: it only makes a client read once more.
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
        self.register_as(user_id, sender, Transport::Sse)
    }

    /// Registers a new connection of `transport` for `user_id`. Beyond
    /// [`MAX_CONNECTIONS_PER_USER`] (whatever their transport) the oldest connections of that user
    /// are removed and told to close.
    pub fn register_as(&self, user_id: u64, sender: SseSender, transport: Transport) -> Connection {
        let connection = Connection {
            id: NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
            sender,
            close: Arc::new(Notify::new()),
            transport,
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
        self.transports_of(user_id)
            .into_iter()
            .map(|(id, sender, _)| (id, sender))
            .collect()
    }

    /// Snapshot of the live connections of `user_id` with their transport.
    pub fn transports_of(&self, user_id: u64) -> Vec<(ConnectionId, SseSender, Transport)> {
        self.online_agents
            .get(&user_id)
            .map(|connections| {
                connections
                    .iter()
                    .map(|connection| {
                        (
                            connection.id,
                            connection.sender.clone(),
                            connection.transport,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Signal pushed when a chat changes: one `data:` line of the SSE stream, or one text frame of the
/// WebSocket (MAIR-657).
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatSignal {
    /// What happened, without the content:
    /// - `NEW_MSG`: a message was posted (signalled to the members but its author);
    /// - `MSG_EDITED`: a message was edited (members but the editor);
    /// - `MSG_DELETED`: a message was deleted (members but the deleting user);
    /// - `CHAT_READ`: the caller acknowledged the chat on another tab or device (only the caller's
    ///   own streams): update the unread badge with `unread_count`.
    ///
    /// For the three message signals, read the change with
    /// `GET /api/v1/{chat_id}/?since=<cursor>`. Ignore an unknown value: new types may be added.
    #[schema(example = "NEW_MSG")]
    pub r#type: String,

    /// Id of the chat concerned, to read with `GET /api/v1/{chat_id}/`.
    #[schema(example = 5)]
    pub chat_id: u64,

    /// The message posted, edited or deleted, or the last message acknowledged by `CHAT_READ`.
    /// Absent only from a signal relayed by a replica older than MAIR-524.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 118, nullable = false)]
    pub message_id: Option<u64>,

    /// `CHAT_READ` only (absent otherwise): messages of the chat still unread by the caller.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 0, nullable = false)]
    pub unread_count: Option<i32>,
}

impl ChatSignal {
    /// The frame announcing `event` to the clients.
    pub fn from_event(event: &ChatEvent) -> Self {
        Self {
            r#type: event.kind.signal_type().to_string(),
            chat_id: event.chat_id,
            message_id: event.message_id,
            unread_count: event.unread_count,
        }
    }
}
