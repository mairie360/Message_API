use crate::endpoints::v1::id::get::view::ReactionView;
use crate::endpoints::v1::id::messages::id::reactions::delete::endpoint::__path_remove_reaction;
use crate::endpoints::v1::id::messages::id::reactions::post::endpoint::__path_add_reaction;
use crate::endpoints::v1::id::messages::id::reactions::post::view::{
    AddReactionView, ReactionsView,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(add_reaction, remove_reaction),
    components(schemas(AddReactionView, ReactionsView, ReactionView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
))]
pub struct ReactionsDoc;
