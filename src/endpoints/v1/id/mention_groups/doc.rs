use crate::endpoints::v1::id::mention_groups::get::endpoint::__path_get_mention_groups;
use crate::endpoints::v1::id::mention_groups::get::view::{MentionGroupView, MentionGroupsView};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_mention_groups),
    components(schemas(MentionGroupsView, MentionGroupView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
))]
pub struct MentionGroupsDoc;
