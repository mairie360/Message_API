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

/// Signal poussé sur le flux SSE quand une conversation change.
#[derive(Serialize, ToSchema)]
pub struct ChatSignal {
    /// Le type d'événement (ex: "NEW_MSG"). Indique qu'il s'est passé quelque chose dans la
    /// conversation, sans porter le contenu : au client de recharger `GET /api/v1/{chat_id}/`.
    #[schema(example = "NEW_MSG")]
    pub r#type: String,

    /// L'identifiant du salon de discussion concerné, à recharger via `GET /api/v1/{chat_id}/`.
    #[schema(example = 5)]
    pub chat_id: u64,
}
