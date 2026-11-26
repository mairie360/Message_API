use crate::endpoints::v1::direct::doc::DirectDoc;
use crate::endpoints::v1::get::endpoint::__path_get_chats;
use crate::endpoints::v1::get::view::GetChatsResultView;
use crate::endpoints::v1::id::doc::IdDoc;
use crate::endpoints::v1::post::endpoint::__path_create_chat;
use crate::endpoints::v1::post::view::{CreateChatResultView, CreateChatView};
use crate::endpoints::v1::search::doc::SearchDoc;
use crate::endpoints::v1::unread::doc::UnreadDoc;
use crate::endpoints::v1::ws::doc::WebSocketDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(create_chat, get_chats),
    components(schemas(CreateChatResultView, CreateChatView, GetChatsResultView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/direct", api = DirectDoc),
    (path = "/search", api = SearchDoc),
    (path = "/{chat_id}", api = IdDoc),
    (path = "/ws", api = WebSocketDoc),
    (path = "/unread", api = UnreadDoc),
    (path = "/", api = Doc)
))]
pub struct V1Doc;
