pub mod doc;
pub mod id;
pub mod post;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/messages")
            .service(post::endpoint::post_message)
            .configure(id::config),
    );
}
