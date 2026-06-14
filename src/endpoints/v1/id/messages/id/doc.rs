use crate::endpoints::v1::id::messages::id::delete::endpoint::__path_delete_message;
use crate::endpoints::v1::id::messages::id::patch::endpoint::__path_patch_message;
use crate::endpoints::v1::id::messages::id::patch::view::PatchMessageView;
use crate::endpoints::v1::id::ChatPathParams;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(delete_message, patch_message),
    components(schemas(ChatPathParams, PatchMessageView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
))]
pub struct IdDoc;
