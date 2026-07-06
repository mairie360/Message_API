use dashmap::DashMap;
use serde::Serialize;
use tokio::sync::broadcast;
use utoipa::ToSchema;

type SseSender = tokio::sync::mpsc::Sender<Result<actix_web::web::Bytes, String>>;

// L'instruction minimaliste que l'API REST va envoyer
#[derive(Clone, serde::Serialize)]
pub struct ChatEvent {
    pub chat_id: u64,
    pub sender_id: u64,
    pub message: String,
}

pub struct AppState {
    // Les agents de la mairie en ligne sur le SSE
    pub online_agents: DashMap<u64, SseSender>,
    // Le canal interne pour que le REST parle au SSE
    pub internal_bus: broadcast::Sender<ChatEvent>,
}

#[derive(Serialize, ToSchema)]
pub struct ChatSignal {
    /// Le type d'événement (ex: "NEW_MSG")
    #[schema(example = "NEW_MSG")]
    pub r#type: String,

    /// L'identifiant unique du salon de discussion concerné
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub chat_id: u64,
}
