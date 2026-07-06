pub mod doc;
pub mod get;
pub mod id;
pub mod post;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/users")
            .service(get::endpoint::get_chat_users)
            .service(post::endpoint::add_users_to_chat)
            .configure(id::config),
    );
}
