use crate::database::search::view::{GlobalSearchItem, GlobalSearchQueryView};
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::search::view::{like_escape, GlobalSearchResultView, SearchQuery};
use crate::endpoints::validation::ValidatedQuery;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    DatabaseError,
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred while accessing the database.")
    }
}

impl ResponseError for SearchError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_global_search(
    state: web::Data<AppState>,
    caller_id: u64,
    query: SearchQuery,
) -> Result<GlobalSearchResultView, SearchError> {
    let view = GlobalSearchQueryView::new(caller_id, &like_escape(query.term()), query.limit());
    let items: Vec<GlobalSearchItem> =
        state
            .get_smart_db()
            .fetch_all(&view)
            .await
            .map_err(|error| {
                unexpected("global_search", error);
                SearchError::DatabaseError
            })?;
    Ok(GlobalSearchResultView { items })
}

#[utoipa::path(
    get,
    path = "",
    summary = "Search the chats and messages (global search)",
    description = "Part of the global search of the shared header: every business API answers \
                   `GET /api/v1/search` with the same shape, and each BFF merges them. Message \
                   returns, among the chats the caller is a non-excluded member of, the **group \
                   chats** whose title contains `q` (`kind = conversation`) and the **messages** \
                   whose content contains `q` (`kind = message`, live messages of users only, \
                   `parent_id` = their chat, `title` = the chat's displayed name, `excerpt` = the \
                   first 120 characters), case-insensitive. Text starting with `q` comes first, \
                   then the most recent. The term is never logged.\n\n\
                   Not cached: the result depends on the term.",
    params(SearchQuery),
    responses(
        (
            status = 200,
            description = "At most `limit` chats and messages, best match first. Empty when nothing matches.",
            body = GlobalSearchResultView,
            example = json!({
                "items": [
                    {
                        "kind": "conversation",
                        "id": 7,
                        "title": "Roads team",
                        "excerpt": null,
                        "parent_id": null,
                        "updated_at": "2026-11-20T09:30:00Z"
                    },
                    {
                        "kind": "message",
                        "id": 1204,
                        "title": "Roads team",
                        "excerpt": "The road marking of Lilac Street is planned for Thursday",
                        "parent_id": 7,
                        "updated_at": "2026-11-19T16:12:00Z"
                    }
                ]
            })
        ),
        (
            status = 400,
            description = "`q` is missing, shorter than 2 or longer than 100 characters once trimmed, or holds a control character; or `limit` is outside 1–20.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `q`: must be at least 2 characters")
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
            description = "Database error while searching.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        )
    ),
    tag = "Search",
    security(
        ("jwt" = [])
    )
)]
#[get("")]
pub async fn global_search(
    caller: AuthenticatedUser,
    state: web::Data<AppState>,
    query: ValidatedQuery<SearchQuery>,
) -> Result<impl Responder, SearchError> {
    let result = trigger_global_search(state, caller.id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
