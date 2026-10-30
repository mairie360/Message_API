//! Redis pub/sub relay of the chat events, so that every replica of the API notifies the SSE
//! connections it holds, whichever replica handled the write.
//!
//! Each replica publishes its events on one channel and subscribes to it; what it receives is
//! pushed on its local `internal_bus`. The channel is `<role>:sse:chat-events`, `<role>` being the
//! key prefix of `mairie360_api_lib` (username of `REDIS_URL`, see `resolve_key_prefix`): the
//! platform's Redis ACL only grants the `message-api` role `PUBLISH` / `SUBSCRIBE` on
//! `&message-api:*`. A local Redis without ACL uses `sse:chat-events`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use mairie360_api_lib::redis::redis_interface::resolve_key_prefix;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use tokio::sync::{broadcast, Mutex};

use crate::sse::state::ChatEvent;

const CHANNEL: &str = "sse:chat-events";
const MAX_BACKOFF: Duration = Duration::from_secs(30);
/// Payload published right after subscribing: receiving it proves the subscription works.
/// redis-rs drops the error reply of `SUBSCRIBE` (an ACL `NOPERM` looks like a success).
const PROBE: &str = "probe";
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

pub struct RedisRelay {
    client: redis::Client,
    channel: String,
    publisher: Mutex<Option<MultiplexedConnection>>,
    subscribed: AtomicBool,
}

impl RedisRelay {
    /// Relay over `redis_url`, or `None` when the URL cannot be parsed (the API then only
    /// notifies its own connections).
    pub fn new(redis_url: &str) -> Option<Arc<Self>> {
        let channel = match resolve_key_prefix(redis_url) {
            Some(prefix) => format!("{prefix}:{CHANNEL}"),
            None => CHANNEL.to_string(),
        };
        Self::with_channel(redis_url, &channel)
    }

    /// Relay over `redis_url` on an explicit channel.
    pub fn with_channel(redis_url: &str, channel: &str) -> Option<Arc<Self>> {
        match redis::Client::open(redis_url) {
            Ok(client) => Some(Arc::new(Self {
                client,
                channel: channel.to_string(),
                publisher: Mutex::new(None),
                subscribed: AtomicBool::new(false),
            })),
            Err(e) => {
                tracing::warn!("SSE relay disabled, invalid Redis URL: {e}");
                None
            }
        }
    }

    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// Whether this replica currently receives the channel.
    pub fn is_subscribed(&self) -> bool {
        self.subscribed.load(Ordering::Acquire)
    }

    /// Publishes `event` on the channel; `false` (logged) when Redis cannot be reached.
    pub async fn publish(&self, event: &ChatEvent) -> bool {
        match serde_json::to_string(event) {
            Ok(payload) => self.publish_payload(payload).await,
            Err(e) => {
                tracing::error!("Failed to serialize the chat event: {e}");
                false
            }
        }
    }

    async fn publish_payload(&self, payload: String) -> bool {
        let mut guard = self.publisher.lock().await;
        if guard.is_none() {
            match self.client.get_multiplexed_async_connection().await {
                Ok(connection) => *guard = Some(connection),
                Err(e) => {
                    tracing::warn!("SSE relay: cannot connect to Redis to publish: {e}");
                    return false;
                }
            }
        }
        let Some(connection) = guard.as_mut() else {
            return false;
        };
        match connection
            .publish::<_, _, i64>(&self.channel, payload)
            .await
        {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("SSE relay: publish failed: {e}");
                // Reconnect on the next event.
                *guard = None;
                false
            }
        }
    }

    /// Subscribes to the channel and pushes every event received on `bus`, forever: a lost
    /// connection is retried with an exponential backoff (1 s up to 30 s).
    pub async fn run(self: Arc<Self>, bus: broadcast::Sender<ChatEvent>) {
        let mut backoff = Duration::from_secs(1);
        loop {
            match self.client.get_async_pubsub().await {
                Ok(mut pubsub) => match pubsub.subscribe(&self.channel).await {
                    Ok(()) => {
                        let mut messages = pubsub.on_message();
                        if self.confirm_subscription(&mut messages, &bus).await {
                            self.subscribed.store(true, Ordering::Release);
                            backoff = Duration::from_secs(1);
                            while let Some(message) = messages.next().await {
                                forward(message.get_payload_bytes(), &bus);
                            }
                            tracing::warn!("SSE relay: Redis subscription lost, reconnecting");
                        } else {
                            tracing::warn!(
                                "SSE relay: no message received on {}, check the Redis ACL \
                                 (`&<role>:*`, `+publish`, `+subscribe`)",
                                self.channel
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("SSE relay: cannot subscribe to {}: {e}", self.channel)
                    }
                },
                Err(e) => tracing::warn!("SSE relay: cannot connect to Redis to subscribe: {e}"),
            }
            self.subscribed.store(false, Ordering::Release);
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(MAX_BACKOFF);
        }
    }
}

impl RedisRelay {
    /// Publishes [`PROBE`] and waits for it (or any message) on `messages`, forwarding the events
    /// received meanwhile.
    async fn confirm_subscription(
        &self,
        messages: &mut (impl futures_util::Stream<Item = redis::Msg> + Unpin),
        bus: &broadcast::Sender<ChatEvent>,
    ) -> bool {
        if !self.publish_payload(PROBE.to_string()).await {
            return false;
        }
        tokio::time::timeout(PROBE_TIMEOUT, async {
            match messages.next().await {
                Some(message) => {
                    forward(message.get_payload_bytes(), bus);
                    true
                }
                None => false,
            }
        })
        .await
        .unwrap_or(false)
    }
}

/// Pushes the event of a channel message on the local bus; probes are skipped.
fn forward(payload: &[u8], bus: &broadcast::Sender<ChatEvent>) {
    if payload == PROBE.as_bytes() {
        return;
    }
    match serde_json::from_slice::<ChatEvent>(payload) {
        // No receiver (listener not started yet) is not an error.
        Ok(event) => {
            let _ = bus.send(event);
        }
        // Without the value serde quotes: a malformed event may carry a message (MAIR-290).
        Err(e) => tracing::warn!(
            "SSE relay: ignored malformed event: {}",
            mairie360_api_lib::error::describe_json_error(&e)
        ),
    }
}
