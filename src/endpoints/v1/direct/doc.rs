use crate::endpoints::v1::direct::post::endpoint::__path_open_direct_chat;
use crate::endpoints::v1::direct::post::view::{OpenDirectChatResultView, OpenDirectChatView};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(open_direct_chat),
    components(schemas(OpenDirectChatView, OpenDirectChatResultView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc)
))]
pub struct DirectDoc;
