use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::endpoints::v1::id::access::{require_chat_access, AccessDenied};

use crate::database::chats::get_chat::view::{GetChatHeaderQueryView, GetChatQueryView, Message};
use crate::database::chats::get_chats::view::GetChatsQueryResultView;
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::get::view::ChatView;
use crate::endpoints::v1::id::get::view::{GetChatQuery, GetChatResultView};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum GetChatError {
    DatabaseError,
    UnknownChat,
    NotFound,
}

impl std::fmt::Display for GetChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetChatError::NotFound => write!(f, "Unknown chat."),
            GetChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            GetChatError::UnknownChat => {
                write!(f, "Unknown chat.")
            }
        }
    }
}

impl ResponseError for GetChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetChatError::NotFound => StatusCode::NOT_FOUND,
            GetChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            GetChatError::UnknownChat => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_get_chat(
    state: web::Data<AppState>,
    chat_id: u64,
    user_id: u64,
    query: GetChatQuery,
) -> Result<GetChatResultView, GetChatError> {
    let limit = query.limit();
    // One extra row tells whether older messages remain.
    let view = GetChatQueryView::new(chat_id, query.before(), limit + 1);
    let header = GetChatHeaderQueryView::new(chat_id, user_id);
    let db = state.get_smart_db();
    // The page and the header of the chat are independent reads: one round trip each, in parallel.
    let (rows, header) = tokio::try_join!(
        async {
            db.fetch_all::<Message, _>(&view).await.map_err(|e| {
                unexpected("get chat", e);
                GetChatError::DatabaseError
            })
        },
        async {
            db.fetch_one::<GetChatsQueryResultView, _>(&header)
                .await
                .map_err(|e| {
                    unexpected("get chat header", e);
                    GetChatError::DatabaseError
                })
        },
    )?;

    Ok(GetChatResultView::from_newest_first(
        ChatView::from(header),
        rows,
        limit,
    ))
}

#[utoipa::path(
    get,
    path = "",
    summary = "Read the messages of a chat",
    description = "Returns the chat itself and one page of its messages, **without modifying** the caller's unread counter. Only \
                   the explicit acknowledgement `POST /api/v1/{chat_id}/read/` lowers it, so polling this route never \
                   marks messages as read.\n\n\
                   **Pagination:** without `before`, the `limit` latest messages (50 by default, 100 at most); the \
                   page is returned oldest first. When `has_more` is `true`, call again with `before=<next_before>` \
                   to get the previous page; the last page has `has_more: false` and `next_before: null`. Message \
                   ids grow in posting order within a chat, so pages never overlap nor skip a message.\n\n\
                   **`chat`** is the chat as the list shows it (`GET /api/v1/`): `name` to display (the other \
                   participant's full name for a direct chat), `kind`, `contact_id`, `member_count` and the caller's \
                   `unread_count`. Together with `GET /api/v1/{chat_id}/users/` for the members' names, it is all \
                   that is needed to open a chat. For an administrator who is not a participant of a direct chat, \
                   `name` holds both names and `contact_id` is `null`.\n\n\
                   **Quotes:** a message that answers another has its `citation` id and a `quoted` object (author and \
                   first 100 characters), also when the quoted message is older than the page.\n\n\
                   Only the members of the chat and administrators may call this route. Anyone else gets the same \
                   `404` as for an unknown chat.",
    responses(
        (
            status = 200,
            description = "The chat and one page of its messages, oldest first.",
            body = GetChatResultView,
            example = json!({
                "chat": { "id": 5, "name": "Service urbanisme", "kind": "group", "contact_id": null, "member_count": 8, "unread_count": 3 },
                "messages": [
                    {
                        "id": 117,
                        "content": "Quelqu'un a des nouvelles du permis de construire de la rue Pasteur ?",
                        "sender_id": 51,
                        "created_at": "2026-09-16T09:05:00Z",
                        "citation": null,
                        "quoted": null
                    },
                    {
                        "id": 118,
                        "content": "La réunion est décalée à 15h.",
                        "sender_id": 42,
                        "created_at": "2026-09-16T09:12:00Z",
                        "citation": 117,
                        "quoted": { "id": 117, "sender_id": 51, "excerpt": "Quelqu'un a des nouvelles du permis de construire de la rue Pasteur ?" }
                    }
                ],
                "has_more": true,
                "next_before": 117
            })
        ),
        (
            status = 400,
            description = "A URL segment is not an integer, or `before` / `limit` is out of range (`before` 1 to 9223372036854775807, `limit` 1 to 100).",
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
    params(
        ChatPathParams,
        GetChatQuery
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[get("/")]
pub async fn get_chat(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    params: web::Path<ChatPathParams>,
    query: ValidatedQuery<GetChatQuery>,
) -> Result<impl Responder, GetChatError> {
    let chat_id = params.chat_id;
    require_chat_access(&state, chat_id, user.id).await?;
    let result = trigger_get_chat(state, chat_id, user.id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for GetChatError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => GetChatError::NotFound,
            AccessDenied::DatabaseError => GetChatError::DatabaseError,
        }
    }
}
