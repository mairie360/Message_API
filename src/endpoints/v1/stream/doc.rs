use crate::{endpoints::v1::stream::endpoint::__path_sse_stream_route, sse::state::ChatSignal};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(sse_stream_route), components(schemas(ChatSignal)))]
pub struct StreamDoc;
