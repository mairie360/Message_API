use crate::endpoints::v1::id::delete::endpoint::__path_delete_chat;
use crate::endpoints::v1::id::get::endpoint::__path_get_chat;
use crate::endpoints::v1::id::messages::doc::MessagesDoc;
use crate::endpoints::v1::id::users::doc::UsersDoc;
use crate::endpoints::v1::id::ChatPathParams;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(delete_chat, get_chat), components(schemas(ChatPathParams)))]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/messages", api = MessagesDoc),
    (path = "/users", api = UsersDoc)
))]
pub struct IdDoc;
