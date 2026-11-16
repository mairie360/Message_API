pub mod delete;
pub mod doc;

/// Path parameters of the routes of a participant.
#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct UsersPathParams {
    /// Id of the conversation.
    #[param(example = 5)]
    #[schema(example = 5)]
    chat_id: u64,
    /// Core API id of the participant.
    #[param(example = 42)]
    #[schema(example = 42)]
    user_id: u64,
}

impl UsersPathParams {
    pub fn new(chat_id: u64, user_id: u64) -> Self {
        Self { chat_id, user_id }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{user_id}").service(delete::endpoint::remove_user_from_chat),
    );
}
