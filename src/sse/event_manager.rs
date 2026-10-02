use crate::{
    database::chats::get_chat_users::view::GetChatMembersQueryView,
    sse::state::{AppState, ChatSignal},
};
use actix_web::web;
use mairie360_api_lib::smart_db::SmartDatabase;
use std::sync::Arc;
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::sync::mpsc::error::TrySendError;

pub async fn start_internal_event_listener(state: Arc<AppState>, smart_db: SmartDatabase) {
    let rx = state.internal_bus.subscribe();
    listen(rx, state, smart_db).await;
}

/// Consumes the internal bus until it is closed. A lagging receiver only loses the events it
/// missed: it is logged and the loop resumes with the oldest event still buffered.
pub async fn listen(
    mut rx: broadcast::Receiver<crate::sse::state::ChatEvent>,
    state: Arc<AppState>,
    smart_db: SmartDatabase,
) {
    loop {
        let event = match rx.recv().await {
            Ok(event) => event,
            Err(RecvError::Lagged(skipped)) => {
                tracing::warn!("SSE listener lagged: {skipped} chat event(s) dropped, resuming");
                continue;
            }
            Err(RecvError::Closed) => break,
        };

        let state_clone = state.clone();
        let smart_db = smart_db.clone();

        // Each event is handled in its own task so a slow query does not block the bus.
        tokio::spawn(async move {
            let view = GetChatMembersQueryView::new(event.chat_id);
            let members: Vec<i32> = match smart_db.fetch_all::<i32, _>(&view).await {
                Ok(members) => members,
                Err(e) => {
                    tracing::error!("Failed to fetch the chat members: {}", e);
                    vec![]
                }
            };

            let signal = ChatSignal {
                r#type: "NEW_MSG".to_string(),
                chat_id: event.chat_id,
            };
            let payload = match serde_json::to_string(&signal) {
                Ok(payload) => payload,
                Err(e) => {
                    tracing::error!("Failed to serialize the chat signal: {}", e);
                    return;
                }
            };

            for user_id in members {
                if user_id == event.sender_id as i32 {
                    continue; // The sender is never notified.
                }
                let user_id = user_id as u64;

                for (connection_id, tx) in state_clone.connections_of(user_id) {
                    let frame = web::Bytes::from(format!("data: {}\n\n", payload));
                    // A full buffer only drops this notification; a closed channel is a dead
                    // connection and is the only thing removed.
                    if let Err(TrySendError::Closed(_)) = tx.try_send(Ok(frame)) {
                        state_clone.remove_connection(user_id, connection_id);
                    }
                }
            }
        });
    }
}
