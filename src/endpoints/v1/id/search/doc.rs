use crate::endpoints::v1::id::search::get::endpoint::__path_search_chat;
use crate::endpoints::v1::id::search::get::view::{
    SearchItem, SearchMessageItem, SearchPersonItem, SearchResultView, SearchType,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(search_chat),
    components(schemas(
        SearchResultView,
        SearchItem,
        SearchMessageItem,
        SearchPersonItem,
        SearchType
    ))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
))]
pub struct SearchDoc;
