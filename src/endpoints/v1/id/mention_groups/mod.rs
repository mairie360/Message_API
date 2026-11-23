//! `/{chat_id}/mention-groups/` (MAIR-1155): the groups a message of the chat may mention.

pub mod doc;
pub mod get;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/mention-groups").service(get::endpoint::get_mention_groups),
    );
}
