//! Connection bookkeeping of the SSE state (no database needed).

use std::time::Duration;

use actix_web::web::Bytes;
use message_api::sse::state::{AppState, ChatEvent, MAX_CONNECTIONS_PER_USER};
use tokio::sync::{broadcast, mpsc};

fn sender() -> mpsc::Sender<Result<Bytes, String>> {
    mpsc::channel(1).0
}

#[tokio::test]
async fn test_opening_too_many_streams_closes_the_oldest() {
    let (bus, _) = broadcast::channel(4);
    let state = AppState::new(bus, None);

    let oldest = state.register(7, sender());
    let mut others = Vec::new();
    for _ in 1..MAX_CONNECTIONS_PER_USER {
        others.push(state.register(7, sender()));
    }
    assert_eq!(state.connections_of(7).len(), MAX_CONNECTIONS_PER_USER);

    let newest = state.register(7, sender());
    let ids: Vec<u64> = state.connections_of(7).iter().map(|(id, _)| *id).collect();
    assert_eq!(ids.len(), MAX_CONNECTIONS_PER_USER);
    assert!(!ids.contains(&oldest.id), "the oldest stream is evicted");
    assert!(ids.contains(&newest.id));

    // The evicted stream is told to close (the permit is kept until it is awaited).
    tokio::time::timeout(Duration::from_secs(1), oldest.close.notified())
        .await
        .expect("the evicted connection is notified");
    // The others are not.
    assert!(
        tokio::time::timeout(Duration::from_millis(100), others[0].close.notified())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_streams_of_other_users_are_not_counted() {
    let (bus, _) = broadcast::channel(4);
    let state = AppState::new(bus, None);
    for user in 0..(MAX_CONNECTIONS_PER_USER as u64 + 2) {
        state.register(user, sender());
    }
    for user in 0..(MAX_CONNECTIONS_PER_USER as u64 + 2) {
        assert_eq!(state.connections_of(user).len(), 1);
    }
}

#[tokio::test]
async fn test_publish_without_relay_uses_the_local_bus() {
    let (bus, mut rx) = broadcast::channel(4);
    let state = AppState::new(bus, None);
    let event = ChatEvent {
        chat_id: 5,
        sender_id: 42,
    };
    state.publish(event.clone()).await;
    assert_eq!(rx.recv().await.unwrap(), event);
}
