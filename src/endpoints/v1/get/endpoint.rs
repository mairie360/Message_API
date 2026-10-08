use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::get_chats::view::{GetChatsQueryResultView, GetChatsQueryView};
use crate::endpoints::error::unexpected;
use crate::endpoints::pagination::PageQuery;
use crate::endpoints::v1::get::view::GetChatsResultView;
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
    page: PageQuery,
) -> Result<GetChatsResultView, GetChatsError> {
    let view = GetChatsQueryView::page(user_id, page.limit(), page.offset());
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
    params(PageQuery),
    summary = "List my chats",
    description = "Returns one page of the chats the user of the JWT is a member of, newest first, with their \
                   number of unread messages in each.\n\n\
                   **Paginated** (`limit` 1 to 100, default 50, and `offset`): `has_more` tells whether another \
                   page follows; ask for it with `offset` increased by `limit`.\n\n\
                   Reading `GET /api/v1/{chat_id}/` does not change `unread_count`: only the explicit \
                   `POST /api/v1/{chat_id}/read/` acknowledgement does.\n\n\
                   A chat without a title has an empty `name`, never `null`.\n\n\
                   `kind` tells a **direct** chat (two agents, opened with `POST /api/v1/direct/`; `contact_id` \
                   is the other participant, even when they hid it, and `name` is empty: show the contact's \
                   name) from a **group** chat (`contact_id` is `null`). A chat the caller hid is not listed.",
    responses(
        (
            status = 200,
            description = "Chats of the caller, newest first.",
            body = GetChatsResultView,
            example = json!({
                "chats": [
                    { "id": 5, "name": "Service urbanisme", "kind": "group", "contact_id": null, "unread_count": 3 },
                    { "id": 9, "name": "", "kind": "direct", "contact_id": 42, "unread_count": 1 },
                    { "id": 8, "name": "Astreinte week-end", "kind": "group", "contact_id": null, "unread_count": 0 }
                ],
                "has_more": false
            })
        ),
        (
            status = 400,
            description = "`limit` not between 1 and 100, or `offset` not between 0 and 2147483647.",
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
    page: ValidatedQuery<PageQuery>,
) -> Result<impl Responder, GetChatsError> {
    let result = trigger_get_chats(state, auth_user.id, page.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
