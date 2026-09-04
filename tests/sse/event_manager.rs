//! Tests d'intégration du listener SSE : il consomme le bus interne, va
//! chercher les membres du chat en base et pousse une frame `data: {...}` aux
//! seuls membres en ligne (hors expéditeur).

use std::sync::Arc;
use std::time::Duration;

use actix_web::web::Bytes;
use dashmap::DashMap;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_users_to_chat::view::AddMembersToChatQueryView, create_chat::view::CreateChatQueryView,
};
use message_api::sse::event_manager::start_internal_event_listener;
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
    let state = Arc::new(AppState {
        online_agents: DashMap::new(),
        internal_bus: bus_tx.clone(),
    });

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
    state.online_agents.insert(1, sender_tx);
    state.online_agents.insert(2, recipient_tx);

    // `ChatEvent` n'implémente pas `Debug` : on ne peut pas `unwrap` le résultat.
    assert!(bus_tx
        .send(ChatEvent {
            chat_id,
            sender_id: 1,
            message: "hello".to_string(),
        })
        .is_ok());

    let frame = tokio::time::timeout(Duration::from_secs(5), recipient_rx.recv())
        .await
        .expect("recipient should receive a frame")
        .expect("channel open")
        .expect("frame ok");
    let text = String::from_utf8(frame.to_vec()).unwrap();
    assert!(text.starts_with("data: "));
    assert!(text.contains("NEW_MSG"));
    assert!(text.contains(&chat_id.to_string()));

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
    state.online_agents.insert(1, agent_tx);

    assert!(bus_tx
        .send(ChatEvent {
            chat_id: 999_999,
            sender_id: 42,
            message: "nobody".to_string(),
        })
        .is_ok());

    assert!(
        tokio::time::timeout(Duration::from_millis(300), agent_rx.recv())
            .await
            .is_err()
    );
}
