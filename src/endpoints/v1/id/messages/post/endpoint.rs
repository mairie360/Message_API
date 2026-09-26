use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::add_message_to_chat::view::PostMessageInChatQueryView;
use crate::endpoints::v1::id::messages::post::view::{PostMessageResultView, PostMessageView};
use crate::endpoints::v1::id::ChatPathParams;
use crate::endpoints::validation::ValidatedJson;
use crate::sse::state::ChatEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum PosteMessageError {
    DatabaseError,
    UnknownChat,
}

impl std::fmt::Display for PosteMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PosteMessageError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            PosteMessageError::UnknownChat => write!(f, "Unknown chat."),
        }
    }
}

impl ResponseError for PosteMessageError {
    fn status_code(&self) -> StatusCode {
        match self {
            PosteMessageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            PosteMessageError::UnknownChat => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_post_message(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::sse::state::AppState>,
    user_id: u64,
    view: PostMessageView,
    chat_id: u64,
) -> Result<PostMessageResultView, PosteMessageError> {
    let view = PostMessageInChatQueryView::new(chat_id, user_id, view.content());
    let chat_event = ChatEvent {
        chat_id,
        sender_id: user_id,
        message: view.message().to_string(),
    };
    let result = state
        .get_smart_db()
        .fetch_scalar::<i64, _>(&view)
        .await
        .map_err(|e| match e {
            ApiLibError::Database(DbError::ForeignKeyViolation(_)) => {
                PosteMessageError::UnknownChat
            }
            _ => PosteMessageError::DatabaseError,
        })?;

    let _ = sse_state.internal_bus.send(chat_event);

    Ok(PostMessageResultView::new(result as u64))
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    summary = "Publier un message",
    description = "Ajoute un message à une conversation et pousse un `ChatSignal` sur le flux SSE \
                   de chaque participant connecté. L'auteur est déduit du JWT, jamais du corps.\n\n\
                   `sitation` est facultatif : il porte l'identifiant du message auquel celui-ci \
                   répond. Attention, le champ est renvoyé systématiquement à `null` par la \
                   lecture `GET /api/v1/{chat_id}/`, qui ne le relit pas encore depuis la base.\n\n\
                   La réponse ne contient que l'identifiant attribué au message.\n\n\
                   Aucun contrôle d'appartenance : tout utilisateur authentifié peut appeler cette route sur \
                   n'importe quelle conversation dont il connaît l'identifiant.",
    responses(
        (
            status = 200,
            description = "Message publié. Le corps contient l'identifiant attribué.",
            body = PostMessageResultView,
            example = json!({ "id": 101 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, `chat_id` not an integer, or `content` breaking its rules: `content` not blank, at most 5000 characters, no `<` / `>`, no control character other than line breaks and tabs.",
            body = String,
            content_type = "text/plain",
            example = json!("Invalid `content`: must not contain `<` or `>`")
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 404,
            description = "No chat matches `chat_id`.",
            body = String,
            content_type = "text/plain",
            example = json!("Unknown chat.")
        ),
        (
            status = 500,
            description = "Erreur de base de données.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = PostMessageView,
        description = "Contenu du message et, éventuellement, le message auquel il répond.",
        example = json!({ "content": "La réunion est décalée à 15h.", "sitation": null })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Messages",
)]
#[post("/")]
pub async fn post_message(
    state: web::Data<AppState>,
    sse_state: web::Data<crate::sse::state::AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<PostMessageView>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, PosteMessageError> {
    let view = view.into_inner();
    let chat_id = params.chat_id;
    let result = trigger_post_message(state, sse_state, auth_user.id, view, chat_id).await?;
    Ok(HttpResponse::Ok().json(result))
}
