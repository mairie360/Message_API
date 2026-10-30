//! Connection bookkeeping of the SSE state (no database needed).

use std::time::Duration;

use actix_web::web::Bytes;
use message_api::sse::state::{
    AppState, ChatEvent, ChatEventKind, ChatSignal, MAX_CONNECTIONS_PER_USER,
};
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
    let event = ChatEvent::message_created(5, 42, 1);
    state.publish(event.clone()).await;
    assert_eq!(rx.recv().await.unwrap(), event);
}

#[test]
fn test_mair_524_an_event_of_an_older_replica_is_a_new_message() {
    // What a replica older than MAIR-524 publishes on the Redis channel.
    let old: ChatEvent = serde_json::from_str(r#"{"chat_id":5,"sender_id":42}"#).unwrap();
    assert_eq!(old.kind, ChatEventKind::MessageCreated);
    assert_eq!((old.message_id, old.unread_count), (None, None));
    assert_eq!(
        serde_json::to_value(ChatSignal::from_event(&old)).unwrap(),
        serde_json::json!({ "type": "NEW_MSG", "chat_id": 5 }),
        "the frame of an old event is unchanged"
    );
    // And what it would read of a new one: the extra fields are ignored by serde.
    let new = serde_json::to_value(ChatEvent::chat_read(5, 42, 118, 3)).unwrap();
    assert_eq!(new["chat_id"], 5);
    assert_eq!(new["sender_id"], 42);
    assert_eq!(new["kind"], "ChatRead");
}

#[test]
fn test_mair_524_signal_types() {
    let cases = [
        (ChatEvent::message_created(5, 1, 117), "NEW_MSG"),
        (ChatEvent::message_edited(5, 1, 117), "MSG_EDITED"),
        (ChatEvent::message_deleted(5, 1, 117), "MSG_DELETED"),
        (ChatEvent::chat_read(5, 1, 117, 0), "CHAT_READ"),
    ];
    for (event, kind) in cases {
        let signal = serde_json::to_value(ChatSignal::from_event(&event)).unwrap();
        assert_eq!(signal["type"], kind);
        assert_eq!(signal["message_id"], 117);
        assert_eq!(signal.get("unread_count").is_some(), kind == "CHAT_READ");
    }
}
