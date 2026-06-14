use crate::endpoints::v1::id::users::id::delete::endpoint::__path_remove_user_from_chat;
use crate::endpoints::v1::id::users::id::UsersPathParams;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(remove_user_from_chat), components(schemas(UsersPathParams)))]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
))]
pub struct IdDoc;
