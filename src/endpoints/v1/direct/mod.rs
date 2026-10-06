pub mod doc;
pub mod post;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/direct").service(post::endpoint::open_direct_chat));
}
