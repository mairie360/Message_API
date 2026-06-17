use crate::endpoints::v1::id::messages::id::doc::IdDoc;
use crate::endpoints::v1::id::messages::post::endpoint::__path_post_message;
use crate::endpoints::v1::id::messages::post::view::{PostMessageResultView, PostMessageView};
use crate::endpoints::v1::id::ChatPathParams;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(post_message),
    components(schemas(ChatPathParams, PostMessageView, PostMessageResultView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{message_id}", api = IdDoc)
))]
pub struct MessagesDoc;
