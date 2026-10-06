use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::open_direct_chat::view::{
    OpenDirectChatQueryResultView, OpenDirectChatQueryView,
};
use crate::database::ids::id_from_sql;
use crate::endpoints::error::{classify, DbFailure};
use crate::endpoints::v1::direct::post::view::{OpenDirectChatResultView, OpenDirectChatView};
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum OpenDirectChatError {
    DatabaseError,
    UnknownContact,
    WithOneself,
}

impl std::fmt::Display for OpenDirectChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenDirectChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            OpenDirectChatError::UnknownContact => write!(f, "`contact_id` is an unknown user."),
            OpenDirectChatError::WithOneself => {
                write!(f, "A direct chat cannot be opened with oneself.")
            }
        }
    }
}

impl ResponseError for OpenDirectChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            OpenDirectChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            OpenDirectChatError::UnknownContact | OpenDirectChatError::WithOneself => {
                StatusCode::BAD_REQUEST
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_open_direct_chat(
    state: web::Data<AppState>,
    user_id: u64,
    view: OpenDirectChatView,
) -> Result<OpenDirectChatResultView, OpenDirectChatError> {
    if view.contact_id() == user_id {
        return Err(OpenDirectChatError::WithOneself);
    }
    // One statement finds or creates the chat and shows it to the caller again.
    let result: OpenDirectChatQueryResultView = state
        .get_smart_db()
        .fetch_one(&OpenDirectChatQueryView::new(user_id, view.contact_id()))
        .await
        .map_err(|e| match classify("open direct chat", e) {
            DbFailure::ForeignKey(_) => OpenDirectChatError::UnknownContact,
            _ => OpenDirectChatError::DatabaseError,
        })?;

    Ok(OpenDirectChatResultView::new(
        id_from_sql(result.id),
        result.created,
    ))
}

#[utoipa::path(
    post,
    path = "",
    summary = "Open the direct chat with an agent",
    description = "Returns **the** direct chat of the caller and `contact_id`, creating it when the pair has none: \
                   there is a single direct chat per pair of agents, whoever opened it, so calling this route \
                   again (or the contact calling it) always answers the same `id` and the history is kept.\n\n\
                   The caller sees the chat again in `GET /api/v1/` if they had hidden it. The contact is not \
                   disturbed: a new chat, or one they hid, only shows up on their side with the next message \
                   (`POST /api/v1/{chat_id}/messages/`), which makes it visible to both participants.\n\n\
                   A direct chat has no title (`name` is empty in `GET /api/v1/`, `kind` is `direct` and \
                   `contact_id` names the other participant). Its participants never change: to talk to more \
                   agents, create a group chat with `POST /api/v1/`.",
    responses(
        (
            status = 200,
            description = "The direct chat of the pair. `created` tells whether this call created it.",
            body = OpenDirectChatResultView,
            example = json!({ "id": 9, "created": false })
        ),
        (
            status = 400,
            description = "Malformed JSON body, missing `contact_id` or outside 1..=2147483647 (`Invalid `contact_id`: …`), `contact_id` is the caller (`A direct chat cannot be opened with oneself.`), or `contact_id` is not a known user (`` `contact_id` is an unknown user.``). Nothing is created.",
            body = String,
            content_type = "text/plain",
            example = json!("A direct chat cannot be opened with oneself.")
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
            description = "Database error (logged with its cause). Nothing is created.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = OpenDirectChatView,
        description = "Core API id of the other participant.",
        example = json!({ "contact_id": 42 })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[post("/")]
pub async fn open_direct_chat(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<OpenDirectChatView>,
) -> Result<impl Responder, OpenDirectChatError> {
    let result = trigger_open_direct_chat(state, auth_user.id, view.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
