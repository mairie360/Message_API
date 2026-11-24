use crate::{endpoints::v1::ws::endpoint::__path_websocket_route, realtime::state::ChatSignal};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(websocket_route), components(schemas(ChatSignal)))]
pub struct WebSocketDoc;
