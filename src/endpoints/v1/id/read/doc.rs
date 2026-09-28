use crate::endpoints::v1::id::read::post::endpoint::__path_acknowledge_read;
use crate::endpoints::v1::id::read::post::view::{AcknowledgeReadResultView, AcknowledgeReadView};
use crate::endpoints::v1::id::ChatPathParams;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(acknowledge_read),
    components(schemas(ChatPathParams, AcknowledgeReadView, AcknowledgeReadResultView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc)
))]
pub struct ReadDoc;
