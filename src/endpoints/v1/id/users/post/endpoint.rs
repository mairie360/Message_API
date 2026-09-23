use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::add_users_to_chat::view::AddMembersToChatQueryView;
use crate::endpoints::v1::id::users::post::view::{AddUsersToChat, AddUsersToChatResultView};
use crate::endpoints::v1::id::ChatPathParams;

#[derive(Debug, Clone, PartialEq)]
pub enum AddUsersToChatError {
    DatabaseError,
    UnknownChat,
    UnknownUser,
    AlreadyMember,
}

impl std::fmt::Display for AddUsersToChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddUsersToChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            AddUsersToChatError::UnknownChat => write!(f, "Unknown chat."),
            AddUsersToChatError::UnknownUser => write!(f, "`users_id` contains an unknown user."),
            AddUsersToChatError::AlreadyMember => {
                write!(f, "A user of `users_id` is already a member of this chat.")
            }
        }
    }
}

impl ResponseError for AddUsersToChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            AddUsersToChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AddUsersToChatError::UnknownChat => StatusCode::NOT_FOUND,
            AddUsersToChatError::UnknownUser => StatusCode::BAD_REQUEST,
            AddUsersToChatError::AlreadyMember => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_add_users_to_chat(
    state: web::Data<AppState>,
    chat_id: u64,
    view: AddUsersToChat,
) -> Result<AddUsersToChatResultView, AddUsersToChatError> {
    let added = view.users_id().to_vec();
    let view = AddMembersToChatQueryView::new(chat_id, added.clone());
    if view.is_empty() {
        return Ok(AddUsersToChatResultView::new(chat_id, Vec::new()));
    }

    state
        .get_smart_db()
        .execute(view)
        .await
        .map_err(|e| match e {
            // Both foreign keys of conversation_members: tell the missing chat from the missing user.
            ApiLibError::Database(DbError::ForeignKeyViolation(message))
                if message.contains("conversation_id") =>
            {
                AddUsersToChatError::UnknownChat
            }
            ApiLibError::Database(DbError::ForeignKeyViolation(_)) => {
                AddUsersToChatError::UnknownUser
            }
            ApiLibError::Database(DbError::UniqueViolation(_)) => {
                AddUsersToChatError::AlreadyMember
            }
            _ => AddUsersToChatError::DatabaseError,
        })?;

    Ok(AddUsersToChatResultView::new(chat_id, added))
}

#[utoipa::path(
    post,
    params(ChatPathParams),
    path = "",
    summary = "Ajouter des participants à une conversation",
    description = "Rattache un ou plusieurs utilisateurs à une conversation en un seul appel. La \
                   conversation apparaît ensuite dans leur `GET /api/v1/`.\n\n\
                   Le champ `added` de la réponse liste les identifiants effectivement rattachés : \
                   le comparer à `users_id` pour repérer ceux qui étaient déjà participants ou qui \
                   n'existent pas.\n\n\
                   Aucun contrôle d'appartenance : tout utilisateur authentifié peut appeler cette route sur \
                   n'importe quelle conversation dont il connaît l'identifiant.",
    responses(
        (
            status = 200,
            description = "Participants ajoutés. `added` liste ceux qui l'ont effectivement été.",
            body = AddUsersToChatResultView,
            example = json!({ "chat_id": 5, "added": [42, 51] })
        ),
        (
            status = 400,
            description = "Malformed JSON body, `chat_id` not an integer, missing `users_id`, or `users_id` containing an unknown user.",
            body = String,
            content_type = "text/plain",
            example = json!("`users_id` contains an unknown user.")
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
            status = 409,
            description = "A user of `users_id` is already a member of the chat; nobody is added.",
            body = String,
            content_type = "text/plain",
            example = json!("A user of `users_id` is already a member of this chat.")
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
        content = AddUsersToChat,
        description = "Identifiants Core API des utilisateurs à rattacher.",
        example = json!({ "users_id": [42, 51] })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Users",
)]
#[post("/")]
pub async fn add_users_to_chat(
    state: web::Data<AppState>,
    _: AuthenticatedUser,
    view: web::Json<AddUsersToChat>,
    params: web::Path<ChatPathParams>,
) -> Result<impl Responder, AddUsersToChatError> {
    let view = view.into_inner();
    let result = trigger_add_users_to_chat(state, params.chat_id, view).await?;
    Ok(HttpResponse::Ok().json(result))
}
