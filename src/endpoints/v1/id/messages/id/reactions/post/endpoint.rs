use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::smart_db::SmartTransaction;
use mairie360_api_lib::state::AppState;

use crate::database::chats::message_reactions::view::AddReactionQueryView;
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::id::access::{begin, commit, AccessDenied, NOT_A_MEMBER_MESSAGE};
use crate::endpoints::v1::id::messages::id::patch::endpoint::SYSTEM_NOTE_MESSAGE;
use crate::endpoints::v1::id::messages::id::reactions::post::view::{
    AddReactionView, ReactionsView,
};
use crate::endpoints::v1::id::messages::id::reactions::{
    prepare_reaction, reactions_of, ReactionDenied,
};
use crate::endpoints::v1::id::messages::id::MessagePathParams;
use crate::endpoints::validation::ValidatedJson;
use crate::realtime::state::ChatEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum AddReactionError {
    DatabaseError,
    UnknownChat,
    NotAMember,
    UnknownMessage,
    SystemNote,
}

impl std::fmt::Display for AddReactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddReactionError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            AddReactionError::UnknownChat => write!(f, "Unknown chat."),
            AddReactionError::NotAMember => f.write_str(NOT_A_MEMBER_MESSAGE),
            AddReactionError::UnknownMessage => write!(f, "Unknown message."),
            AddReactionError::SystemNote => f.write_str(SYSTEM_NOTE_MESSAGE),
        }
    }
}

impl ResponseError for AddReactionError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddReactionError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AddReactionError::UnknownChat | AddReactionError::UnknownMessage => {
                StatusCode::NOT_FOUND
            }
            AddReactionError::NotAMember => StatusCode::FORBIDDEN,
            AddReactionError::SystemNote => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_add_reaction(
    tx: &mut SmartTransaction,
    params: &MessagePathParams,
    user_id: u64,
    view: &AddReactionView,
) -> Result<ReactionsView, AddReactionError> {
    prepare_reaction(tx, params.chat_id(), params.message_id(), user_id).await?;
    tx.execute(&AddReactionQueryView::new(
        params.message_id(),
        user_id,
        view.emoji(),
    ))
    .await
    .map_err(|e| {
        unexpected("add reaction", e);
        AddReactionError::DatabaseError
    })?;
    Ok(ReactionsView::new(
        reactions_of(tx, params.message_id()).await?,
    ))
}

#[utoipa::path(
    post,
    path = "",
    summary = "React to a message",
    description = "Adds the caller's reaction `emoji` to a message (MAIR-1155) and answers the reactions of the message. \
                   Adding a reaction the caller already made changes nothing (same answer). A member may react with \
                   several emojis.\n\n \
                   Once committed, a `MSG_REACTED` `ChatSignal` (with `message_id`) is pushed to the other members; the \
                   message comes again in `GET /api/v1/{chat_id}/?since=` with its new `reactions`.\n\n \
                   Only the members of the chat may react: an administrator who is not a member gets `403`, anyone \
                   else the same `404` as for an unknown chat. A deleted message answers `404`, a system note `400`.",
    params(MessagePathParams),
    request_body(
        content = AddReactionView,
        description = "The emoji.",
        example = json!({ "emoji": "👍" })
    ),
    responses(
        (
            status = 200,
            description = "Reaction added (or already there): the reactions of the message.",
            body = ReactionsView,
            example = json!({ "reactions": [{ "emoji": "👍", "count": 2, "user_ids": [42, 51] }] })
        ),
        (
            status = 400,
            description = "Malformed JSON body, a URL segment not an integer, `emoji` empty, longer than 32 characters or holding whitespace or a control character, or the message is a system note (`A system note cannot be changed.`).",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `emoji`: must not contain whitespace nor control characters")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 403,
            description = "The caller is an administrator who is not a member of the chat.",
            body = String,
            content_type = "text/plain",
            example = json!("Administrators may only read and moderate a chat they are not a member of.")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id` or the caller is neither a member nor an administrator (`Unknown chat.`), or no live message `message_id` in this chat (`Unknown message.`).",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown message.")
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
    tag = "Messages",
)]
#[post("/")]
pub async fn add_reaction(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::realtime::state::AppState>,
    auth_user: AuthenticatedUser,
    params: web::Path<MessagePathParams>,
    view: ValidatedJson<AddReactionView>,
) -> Result<impl Responder, AddReactionError> {
    let mut tx = begin(&state).await?;
    let result = trigger_add_reaction(&mut tx, &params, auth_user.id, &view.into_inner()).await?;
    commit(tx).await?;
    sse_state
        .publish(ChatEvent::message_reacted(
            params.chat_id(),
            auth_user.id,
            params.message_id(),
        ))
        .await;
    Ok(HttpResponse::Ok().json(result))
}

impl From<AccessDenied> for AddReactionError {
    fn from(denied: AccessDenied) -> Self {
        match denied {
            AccessDenied::NotFound => AddReactionError::UnknownChat,
            AccessDenied::DatabaseError => AddReactionError::DatabaseError,
        }
    }
}

impl From<ReactionDenied> for AddReactionError {
    fn from(denied: ReactionDenied) -> Self {
        match denied {
            ReactionDenied::UnknownChat => AddReactionError::UnknownChat,
            ReactionDenied::NotAMember => AddReactionError::NotAMember,
            ReactionDenied::UnknownMessage => AddReactionError::UnknownMessage,
            ReactionDenied::SystemNote => AddReactionError::SystemNote,
            ReactionDenied::DatabaseError => AddReactionError::DatabaseError,
        }
    }
}
