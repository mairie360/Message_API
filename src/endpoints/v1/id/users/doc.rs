use crate::endpoints::v1::id::users::get::endpoint::__path_get_chat_users;
use crate::endpoints::v1::id::users::get::view::GetUsersView;
use crate::endpoints::v1::id::users::id::doc::IdDoc;
use crate::endpoints::v1::id::users::post::endpoint::__path_add_users_to_chat;
use crate::endpoints::v1::id::users::post::view::AddUsersToChat;
use crate::endpoints::v1::id::ChatPathParams;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_chat_users, add_users_to_chat),
    components(schemas(ChatPathParams, AddUsersToChat, GetUsersView))
)]
struct Doc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/", api = Doc),
    (path = "/{user_id}", api = IdDoc)
))]
pub struct UsersDoc;
