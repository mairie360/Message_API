use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::mention_groups::view::{MentionGroupRow, MentionGroupsQueryView};
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::id::access::{require_chat_access, AccessDenied};
use crate::endpoints::v1::id::mention_groups::get::view::{
    MentionGroupView, MentionGroupsQuery, MentionGroupsView,
};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum GetMentionGroupsError {
    DatabaseError,
    NotFound,
}

impl std::fmt::Display for GetMentionGroupsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetMentionGroupsError::NotFound => write!(f, "Unknown chat."),
            GetMentionGroupsError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
        }
    }
}

impl ResponseError for GetMentionGroupsError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetMentionGroupsError::NotFound => StatusCode::NOT_FOUND,
            GetMentionGroupsError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_mention_groups(
    state: web::Data<AppState>,
    chat_id: u64,
    query: MentionGroupsQuery,
) -> Result<MentionGroupsView, GetMentionGroupsError> {
    let rows: Vec<MentionGroupRow> = state
        .get_smart_db()
        .fetch_all(&MentionGroupsQueryView::new(chat_id, query.q()))
        .await
        .map_err(|e| {
            unexpected("mention groups", e);
            GetMentionGroupsError::DatabaseError
        })?;
    Ok(MentionGroupsView::new(
        rows.into_iter().map(MentionGroupView::from).collect(),
    ))
}

#[utoipa::path(
    get,
    params(ChatPathParams, MentionGroupsQuery),
    path = "",
    summary = "List the groups a message may mention",
    description = "Returns the groups (Core API groups) at least one member of the chat belongs to, whose name contains \
                   `q` (case-insensitive; every such group without `q`), by name, at most 20 (MAIR-1155). These are \
                   the ids a message accepts in `group_mentions`.\n\n\
                   Open to the members of the chat and to administrators; anyone else gets the same `404` as for an \
                   unknown chat.",
    responses(
        (
            status = 200,
            description = "The groups, by name (empty when none matches).",
            body = MentionGroupsView,
            example = json!({ "groups": [{ "id": 3, "name": "Urban planning" }] })
        ),
        (
            status = 400,
            description = "`chat_id` is not an integer, or `q` longer than 64 characters or holding a control character.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `q`: must be at most 64 characters")
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
pub async fn get_mention_groups(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
    query: ValidatedQuery<MentionGroupsQuery>,
) -> Result<impl Responder, GetMentionGroupsError> {
    let chat_id = params.chat_id();
    require_chat_access(&state, chat_id, auth_user.id).await?;
    let result = trigger_get_mention_groups(state, chat_id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetMentionGroupsError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetMentionGroupsError::NotFound,
            AccessDenied::DatabaseError => GetMentionGroupsError::DatabaseError,
        }
    }
}
