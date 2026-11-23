use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::search::view::{
    MessageMatch, PersonMatch, SearchChatQueryView, SearchPage, SearchTarget,
};
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::id::access::{require_chat_access, AccessDenied};
use crate::endpoints::v1::id::search::get::view::{
    SearchItem, SearchMessageItem, SearchPersonItem, SearchQuery, SearchResultView, SearchType,
};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum SearchChatError {
    DatabaseError,
    NotFound,
}

impl std::fmt::Display for SearchChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchChatError::NotFound => write!(f, "Unknown chat."),
            SearchChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for SearchChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            SearchChatError::NotFound => StatusCode::NOT_FOUND,
            SearchChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_search_chat(
    state: web::Data<AppState>,
    chat_id: u64,
    query: SearchQuery,
) -> Result<SearchResultView, SearchChatError> {
    let search_type = query.search_type();
    let target = match search_type {
        SearchType::Messages => SearchTarget::Messages,
        SearchType::People => SearchTarget::People,
        // No attachments yet: nothing to read.
        SearchType::Files => return Ok(SearchResultView::new(search_type, Vec::new(), 0, 0)),
    };
    let view = SearchChatQueryView::new(target, chat_id, query.q(), query.limit(), query.offset());
    let db = state.get_smart_db();
    let (items, total) = match target {
        SearchTarget::Messages => {
            let page: SearchPage<MessageMatch> = db.fetch_one(&view).await.map_err(|e| {
                unexpected("search chat messages", e);
                SearchChatError::DatabaseError
            })?;
            let items = page
                .items
                .into_iter()
                .map(|row| SearchItem::Message(SearchMessageItem::from(row)))
                .collect();
            (items, page.total)
        }
        SearchTarget::People => {
            let page: SearchPage<PersonMatch> = db.fetch_one(&view).await.map_err(|e| {
                unexpected("search chat people", e);
                SearchChatError::DatabaseError
            })?;
            let items = page
                .items
                .into_iter()
                .map(|row| SearchItem::Person(SearchPersonItem::from(row)))
                .collect();
            (items, page.total)
        }
    };
    Ok(SearchResultView::new(
        search_type,
        items,
        query.offset(),
        u32::try_from(total).unwrap_or_default(),
    ))
}

#[utoipa::path(
    get,
    params(ChatPathParams, SearchQuery),
    path = "",
    summary = "Search in a chat",
    description = "Searches inside one chat (MAIR-1155), case-insensitively, `q` anywhere in the text (`%`, `_` and `\\` \
                   only match themselves; accents are not folded).\n\n\
                   - `type=messages` (default): the messages of the members whose content contains `q`, newest first \
                   (deleted messages and system notes are left out): `{id, sender_id, created_at, content}`.\n\
                   - `type=people`: the members whose first name, last name or full name (both orders) contains `q`, \
                   by last name then first name: `{id, first_name, last_name}`.\n\
                   - `type=files`: always empty, the chats hold no files yet.\n\n\
                   **Paginated** (`limit` 1 to 50, default 20, and `offset`): `total` counts every match, `has_more` \
                   tells whether another page follows.\n\n\
                   Open to the members of the chat and to administrators; anyone else gets the same `404` as for an \
                   unknown chat.",
    responses(
        (
            status = 200,
            description = "One page of the matches.",
            body = SearchResultView,
            example = json!({
                "type": "messages",
                "items": [{ "id": 117, "sender_id": 51, "created_at": "2026-09-16T09:05:00Z", "content": "Any news about the building permit for rue Pasteur?" }],
                "has_more": false,
                "total": 1
            })
        ),
        (
            status = 400,
            description = "`chat_id` is not an integer, `q` missing, blank, longer than 150 characters or holding a control character, `type` not one of `messages`, `people`, `files`, `limit` not between 1 and 50, or `offset` not between 0 and 2147483647.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `q`: must not be empty")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`, or the caller is neither one of its members nor an administrator (both cases are deliberately indistinguishable).",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown chat.")
        ),
        (
            status = 500,
            description = "Database error.",
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
pub async fn search_chat(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
    query: ValidatedQuery<SearchQuery>,
) -> Result<impl Responder, SearchChatError> {
    let chat_id = params.chat_id();
    require_chat_access(&state, chat_id, auth_user.id).await?;
    let result = trigger_search_chat(state, chat_id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for SearchChatError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => SearchChatError::NotFound,
            AccessDenied::DatabaseError => SearchChatError::DatabaseError,
        }
    }
}
