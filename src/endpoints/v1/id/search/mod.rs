//! `/{chat_id}/search/` (MAIR-1155): search inside a chat.

pub mod doc;
pub mod get;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/search").service(get::endpoint::search_chat));
}
