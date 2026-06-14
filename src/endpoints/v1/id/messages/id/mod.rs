pub mod delete;
pub mod doc;
pub mod patch;

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct MessagePathParams {
    chat_id: u64,
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
