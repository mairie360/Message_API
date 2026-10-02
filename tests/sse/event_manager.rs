//! Tests d'intégration du listener SSE : il consomme le bus interne, va
//! chercher les membres du chat en base et pousse une frame `data: {...}` aux
//! seuls membres en ligne (hors expéditeur).

use std::sync::Arc;
use std::time::Duration;

use actix_web::web::Bytes;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_users_to_chat::view::AddMembersToChatQueryView, create_chat::view::CreateChatQueryView,
};
use message_api::sse::event_manager::{listen, start_internal_event_listener};
use message_api::sse::state::{AppState, ChatEvent};
use serial_test::serial;
use tokio::sync::{broadcast, mpsc};

use crate::common::get_smart_db;

type SseChannel = (
    mpsc::Sender<Result<Bytes, String>>,
    mpsc::Receiver<Result<Bytes, String>>,
);

async fn spawn_listener(db_url: &str) -> (Arc<AppState>, broadcast::Sender<ChatEvent>) {
    let db = get_smart_db(db_url).await;
    let (bus_tx, _) = broadcast::channel(16);
    let state = Arc::new(AppState::new(bus_tx.clone(), None));

    tokio::spawn(start_internal_event_listener(state.clone(), db));
    // Laisse le temps au listener de s'abonner au bus avant le premier `send`.
    tokio::time::sleep(Duration::from_millis(200)).await;

    (state, bus_tx)
}

#[tokio::test]
#[serial]
async fn test_event_manager_notifies_online_members_except_sender() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("SSE Chat", None))
        .await
        .unwrap() as u64;
    db.execute(AddMembersToChatQueryView::new(chat_id, vec![1, 2]))
        .await
        .unwrap();

    let (state, bus_tx) = spawn_listener(host).await;

    // L'expéditeur (1) et le destinataire (2) sont tous les deux en ligne.
    let (sender_tx, mut sender_rx): SseChannel = mpsc::channel(16);
    let (recipient_tx, mut recipient_rx): SseChannel = mpsc::channel(16);
    state.register_connection(1, sender_tx);
    state.register_connection(2, recipient_tx);

    // `ChatEvent` n'implémente pas `Debug` : on ne peut pas `unwrap` le résultat.
    assert!(bus_tx
        .send(ChatEvent {
            chat_id,
            sender_id: 1,
        })
        .is_ok());

    let frame = tokio::time::timeout(Duration::from_secs(5), recipient_rx.recv())
        .await
        .expect("recipient should receive a frame")
        .expect("channel open")
        .expect("frame ok");
    let text = String::from_utf8(frame.to_vec()).unwrap();
    assert!(text.starts_with("data: "));
    let payload: serde_json::Value =
        serde_json::from_str(text.trim_start_matches("data: ").trim()).unwrap();
    assert_eq!(payload["type"], "NEW_MSG");
    // `chat_id` is a JSON number, not a string.
    assert_eq!(payload["chat_id"], serde_json::json!(chat_id));

    // L'expéditeur ne doit jamais recevoir sa propre notification.
    assert!(
        tokio::time::timeout(Duration::from_millis(300), sender_rx.recv())
            .await
            .is_err()
    );
}

#[tokio::test]
#[serial]
async fn test_event_manager_ignores_unknown_chat() {
    let (_container, host) = get_shared_db().await;

    let (state, bus_tx) = spawn_listener(host).await;

    let (agent_tx, mut agent_rx): SseChannel = mpsc::channel(16);
    state.register_connection(1, agent_tx);

    assert!(bus_tx
        .send(ChatEvent {
            chat_id: 999_999,
            sender_id: 42,
        })
        .is_ok());

    assert!(
        tokio::time::timeout(Duration::from_millis(300), agent_rx.recv())
            .await
            .is_err()
    );
}

#[tokio::test]
#[serial]
async fn test_event_manager_notifies_every_connection_of_a_user() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("SSE Multi", None))
        .await
        .unwrap() as u64;
    db.execute(AddMembersToChatQueryView::new(chat_id, vec![1, 2]))
        .await
        .unwrap();

    let (state, bus_tx) = spawn_listener(host).await;

    let (first_tx, mut first_rx): SseChannel = mpsc::channel(16);
    let (second_tx, mut second_rx): SseChannel = mpsc::channel(16);
    let first_id = state.register_connection(2, first_tx);
    state.register_connection(2, second_tx);

    // The first tab closes: only its own sender may be dropped.
    state.remove_connection(2, first_id);
    assert_eq!(state.connections_of(2).len(), 1);

    assert!(bus_tx
        .send(ChatEvent {
            chat_id,
            sender_id: 1,
        })
        .is_ok());

    tokio::time::timeout(Duration::from_secs(5), second_rx.recv())
        .await
        .expect("the remaining connection should still be notified")
        .expect("channel open")
        .expect("frame ok");
    assert!(
        tokio::time::timeout(Duration::from_millis(300), first_rx.recv())
            .await
            .map(|frame| frame.is_none())
            .unwrap_or(true)
    );
}

#[tokio::test]
#[serial]
async fn test_event_manager_prunes_only_closed_connections() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("SSE Prune", None))
        .await
        .unwrap() as u64;
    db.execute(AddMembersToChatQueryView::new(chat_id, vec![1, 2]))
        .await
        .unwrap();

    let (state, bus_tx) = spawn_listener(host).await;

    let (dead_tx, dead_rx): SseChannel = mpsc::channel(16);
    let (live_tx, mut live_rx): SseChannel = mpsc::channel(16);
    state.register_connection(2, dead_tx);
    state.register_connection(2, live_tx);
    drop(dead_rx);

    assert!(bus_tx
        .send(ChatEvent {
            chat_id,
            sender_id: 1,
        })
        .is_ok());

    tokio::time::timeout(Duration::from_secs(5), live_rx.recv())
        .await
        .expect("live connection notified")
        .expect("channel open")
        .expect("frame ok");
    assert_eq!(state.connections_of(2).len(), 1);
}

#[tokio::test]
#[serial]
async fn test_event_manager_survives_a_lagged_receiver() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("SSE Lagged", None))
        .await
        .unwrap() as u64;
    db.execute(AddMembersToChatQueryView::new(chat_id, vec![1, 2]))
        .await
        .unwrap();

    // A tiny bus overflowed before the listener reads anything: its first `recv` is `Lagged`.
    let (bus_tx, _keep_alive) = broadcast::channel(2);
    let state = Arc::new(AppState::new(bus_tx.clone(), None));
    let rx = bus_tx.subscribe();
    for _ in 0..5 {
        let _ = bus_tx.send(ChatEvent {
            chat_id: 999_999,
            sender_id: 42,
        });
    }
    tokio::spawn(listen(rx, state.clone(), db));

    let (recipient_tx, mut recipient_rx): SseChannel = mpsc::channel(16);
    state.register_connection(2, recipient_tx);

    assert!(bus_tx
        .send(ChatEvent {
            chat_id,
            sender_id: 1,
        })
        .is_ok());

    tokio::time::timeout(Duration::from_secs(5), recipient_rx.recv())
        .await
        .expect("listener should keep running after a lag")
        .expect("channel open")
        .expect("frame ok");
}
