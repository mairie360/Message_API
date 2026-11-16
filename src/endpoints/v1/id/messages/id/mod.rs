pub mod delete;
pub mod doc;
pub mod patch;

/// Path parameters of the routes of a message.
#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct MessagePathParams {
    /// Id of the conversation.
    #[param(example = 5)]
    #[schema(example = 5)]
    chat_id: u64,
    /// Id of the message, in the conversation of the path.
    #[param(example = 118)]
    #[schema(example = 118)]
    message_id: u64,
}

impl MessagePathParams {
    pub fn new(chat_id: u64, message_id: u64) -> Self {
        Self {
            chat_id,
            message_id,
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }

    pub fn message_id(&self) -> u64 {
        self.message_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{message_id}")
            .service(delete::endpoint::delete_message)
            .service(patch::endpoint::patch_message),
    );
}
