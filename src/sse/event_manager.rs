use crate::database::ids::id_from_sql;
use crate::{
    database::chats::get_chat_users::view::GetChatMembersQueryView,
    sse::state::{AppState, ChatEventKind, ChatSignal},
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
            let payload = match serde_json::to_string(&ChatSignal::from_event(&event)) {
                Ok(payload) => payload,
                Err(e) => {
                    tracing::error!("Failed to serialize the chat signal: {}", e);
                    return;
                }
            };

            // A read acknowledgement only concerns the reader: their other tabs and devices
            // update their unread badge. Everything else goes to the members but its author.
            if event.kind == ChatEventKind::ChatRead {
                push(&state_clone, event.sender_id, &payload);
                return;
            }

            let view = GetChatMembersQueryView::new(event.chat_id);
            let members: Vec<i32> = match smart_db.fetch_all::<i32, _>(&view).await {
                Ok(members) => members,
                Err(e) => {
                    tracing::error!("Failed to fetch the chat members: {}", e);
                    vec![]
                }
            };

            for user_id in members {
                let user_id = id_from_sql(user_id);
                if user_id == event.sender_id {
                    continue; // The author of a change is never notified of it.
                }
                push(&state_clone, user_id, &payload);
            }
        });
    }
}

/// Pushes one `data:` frame to every connection of `user_id` on this replica.
fn push(state: &AppState, user_id: u64, payload: &str) {
    for (connection_id, tx) in state.connections_of(user_id) {
        let frame = web::Bytes::from(format!("data: {}\n\n", payload));
        // A full buffer only drops this notification; a closed channel is a dead connection and
        // is the only thing removed.
        if let Err(TrySendError::Closed(_)) = tx.try_send(Ok(frame)) {
            state.remove_connection(user_id, connection_id);
        }
    }
}
