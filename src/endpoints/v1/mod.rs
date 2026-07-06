pub mod doc;
pub mod get;
pub mod id;
pub mod post;
pub mod stream;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/v1")
            .service(get::endpoint::get_chats)
            .service(post::endpoint::create_chat)
            .service(stream::endpoint::sse_stream_route)
            .configure(id::config),
    );
}
