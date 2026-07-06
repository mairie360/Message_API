pub mod delete;
pub mod doc;
pub mod get;
pub mod messages;
pub mod users;

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Path)]
pub struct ChatPathParams {
    chat_id: u64,
}

impl ChatPathParams {
    pub fn new(chat_id: u64) -> Self {
        Self { chat_id }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/{chat_id}")
            .service(delete::endpoint::delete_chat)
            .service(get::endpoint::get_chat)
            .configure(messages::config)
            .configure(users::config),
    );
}
