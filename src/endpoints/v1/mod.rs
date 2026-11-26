pub mod direct;
pub mod doc;
pub mod get;
pub mod id;
pub mod post;
pub mod search;
pub mod unread;
pub mod ws;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/v1")
            .service(get::endpoint::get_chats)
            .service(post::endpoint::create_chat)
            .service(ws::endpoint::websocket_route)
            // Before `id`: `/{chat_id}` would take `/direct` or `/search` and refuse them as non-integer
            // ids.
            .configure(direct::config)
            .service(actix_web::web::scope("/search").configure(search::config))
            .configure(unread::config)
            .configure(id::config),
    );
}
