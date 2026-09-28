pub mod doc;
pub mod post;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(actix_web::web::scope("/read").service(post::endpoint::acknowledge_read));
}
