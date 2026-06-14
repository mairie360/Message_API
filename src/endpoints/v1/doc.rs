use crate::endpoints::v1::get::endpoint::__path_get_chats;
use crate::endpoints::v1::get::view::GetChatsResultView;
use crate::endpoints::v1::id::doc::IdDoc;
use crate::endpoints::v1::post::endpoint::__path_create_chat;
use crate::endpoints::v1::post::view::{CreateChatResultView, CreateChatView};
use crate::endpoints::v1::stream::doc::StreamDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(create_chat, get_chats),
    components(schemas(CreateChatResultView, CreateChatView, GetChatsResultView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/{chat_id}", api = IdDoc),
    (path = "/stream", api = StreamDoc),
    (path = "/", api = Doc)
))]
pub struct V1Doc;
