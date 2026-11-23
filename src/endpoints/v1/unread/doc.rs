use crate::endpoints::v1::unread::get::endpoint::__path_get_unread_total;
use crate::endpoints::v1::unread::get::view::UnreadTotalView;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(get_unread_total), components(schemas(UnreadTotalView)))]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc)
))]
pub struct UnreadDoc;
