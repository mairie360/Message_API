pub mod direct;
pub mod doc;
pub mod get;
pub mod id;
pub mod post;
pub mod stream;
pub mod unread;
pub mod ws;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/v1")
            .service(get::endpoint::get_chats)
            .service(post::endpoint::create_chat)
            .service(stream::endpoint::sse_stream_route)
            .service(ws::endpoint::websocket_route)
            // Before `id`: `/{chat_id}` would take `/direct` and refuse it as a non-integer id.
            .configure(direct::config)
            .configure(unread::config)
            .configure(id::config),
    );
}
