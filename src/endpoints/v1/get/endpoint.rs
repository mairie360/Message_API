use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::get_chats::view::{GetChatsQueryResultView, GetChatsQueryView};
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::get::view::{ChatListQuery, GetChatsResultView};
use crate::endpoints::validation::ValidatedQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum GetChatsError {
    DatabaseError,
}

impl std::fmt::Display for GetChatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetChatsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetChatsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetChatsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_chats(
    state: web::Data<AppState>,
    user_id: u64,
    query: ChatListQuery,
) -> Result<GetChatsResultView, GetChatsError> {
    let page = query.page();
    let view = GetChatsQueryView::page(user_id, page.limit(), page.offset(), query.search());
    let result: Vec<GetChatsQueryResultView> =
        state.get_smart_db().fetch_all(&view).await.map_err(|e| {
            unexpected("get chats", e);
            GetChatsError::DatabaseError
        })?;

    Ok(GetChatsResultView::from_rows(result, page.limit()))
}

#[utoipa::path(
    get,
    path = "",
    params(ChatListQuery),
    summary = "List my chats",
    description = "Returns one page of the chats the user of the JWT is a member of, newest first, with their \
                   name, member count and number of unread messages in each. One call is enough to display \
                   the list: the ids and names of the members of a chat are only read when it is opened, with \
                   `GET /api/v1/{chat_id}/users/`.\n\n\
                   **Paginated** (`limit` 1 to 100, default 50, and `offset`): `has_more` tells whether another \
                   page follows; ask for it with `offset` increased by `limit`.\n\n\
                   Reading `GET /api/v1/{chat_id}/` does not change `unread_count`: only the explicit \
                   `POST /api/v1/{chat_id}/read/` acknowledgement does.\n\n\
                   `name` is ready to display: the title of a group chat (empty when it has none, never `null`), \
                   the full name of the other participant for a direct chat.\n\n\
                   `kind` tells a **direct** chat (two agents, opened with `POST /api/v1/direct/`; `contact_id` \
                   is the other participant, even when they hid it) from a **group** chat (`contact_id` is \
                   `null`). A chat the caller hid is not listed.\n\n\
                   **Search:** `search` keeps the chats whose name contains the text, or in which a member other \
                   than the caller has a first name, last name or full name containing it. \"autoroute\" finds \
                   the chat \"Projet autoroute\"; \"Xavier Bertrand\" finds the direct chat with him and the group \
                   chats he belongs to. The pagination applies to the filtered list.",
    responses(
        (
            status = 200,
            description = "Chats of the caller, newest first.",
            body = GetChatsResultView,
            example = json!({
                "chats": [
                    { "id": 5, "name": "Service urbanisme", "kind": "group", "contact_id": null, "member_count": 8, "unread_count": 3 },
                    { "id": 9, "name": "Xavier Bertrand", "kind": "direct", "contact_id": 42, "member_count": 2, "unread_count": 1 },
                    { "id": 8, "name": "Astreinte week-end", "kind": "group", "contact_id": null, "member_count": 4, "unread_count": 0 }
                ],
                "has_more": false
            })
        ),
        (
            status = 400,
            description = "`limit` not between 1 and 100, `offset` not between 0 and 2147483647, or `search` longer than 150 characters or containing a control character.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `limit`: must be between 1 and 100")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 500,
            description = "Database error (logged with its cause).",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[get("/")]
pub async fn get_chats(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    query: ValidatedQuery<ChatListQuery>,
) -> Result<impl Responder, GetChatsError> {
    let result = trigger_get_chats(state, auth_user.id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
