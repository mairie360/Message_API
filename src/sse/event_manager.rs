use crate::{database::chats::get_chat_users::view::GetChatMembersQueryView, sse::state::AppState};
use actix_web::web;
use mairie360_api_lib::smart_db::SmartDatabase;
use std::sync::Arc;

pub async fn start_internal_event_listener(
    state: Arc<AppState>,
    smart_db: SmartDatabase, // Accès DB/cache de la mairie
) {
    // On s'abonne au bus interne
    let mut rx = state.internal_bus.subscribe();

    // Cette boucle tourne en tâche de fond à l'infini
    while let Ok(event) = rx.recv().await {
        let state_clone = state.clone();
        let smart_db = smart_db.clone();

        // On traite chaque événement dans une sous-tâche pour ne pas bloquer le bus
        tokio::spawn(async move {
            // 1. Le SSE va chercher en DB qui doit recevoir les messages pour ce chat
            let view = GetChatMembersQueryView::new(event.chat_id);
            let members: Vec<i32> = match smart_db.fetch_all::<i32, _>(&view).await {
                Ok(members) => members,
                Err(e) => {
                    eprintln!("Erreur lors de la récupération des membres du chat: {}", e);
                    vec![]
                }
            };

            // 2. Diffusion ciblée aux agents en ligne
            for user_id in members {
                if user_id == event.sender_id as i32 {
                    continue;
                } // Pas de notification à l'expéditeur

                if let Some(tx) = state_clone.online_agents.get(&(user_id as u64)) {
                    let payload =
                        format!(r#"{{"type": "NEW_MSG", "chat_id": "{}"}}"#, event.chat_id);
                    let sse_data = format!("data: {}\n\n", payload);

                    // Envoi au navigateur de l'agent
                    let _ = tx.try_send(Ok(web::Bytes::from(sse_data)));
                }
            }
        });
    }
}
