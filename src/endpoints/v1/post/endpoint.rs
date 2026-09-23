use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::security::AuthenticatedUser;
use mairie360_api_lib::state::AppState;

use crate::database::chats::add_users_to_chat::view::AddMembersToChatQueryView;
use crate::database::chats::create_chat::view::CreateChatQueryView;
use crate::database::chats::delete_chat::view::DeleteChatQueryView;
use crate::endpoints::v1::post::view::{CreateChatResultView, CreateChatView};
use crate::endpoints::validation::ValidatedJson;

#[derive(Debug, Clone, PartialEq)]
pub enum CreateChatError {
    DatabaseError,
    UnknownMember,
}

impl std::fmt::Display for CreateChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateChatError::DatabaseError => {
                write!(f, "An error occurred while accessing the database.")
            }
            CreateChatError::UnknownMember => {
                write!(f, "`members` contains an unknown user.")
            }
        }
    }
}

impl ResponseError for CreateChatError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateChatError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            CreateChatError::UnknownMember => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

async fn trigger_create_chat(
    state: web::Data<AppState>,
    user_id: u64,
    view: CreateChatView,
) -> Result<CreateChatResultView, CreateChatError> {
    let db = state.get_smart_db();

    let query_view = CreateChatQueryView::new(view.name(), None);
    let result = db
        .fetch_scalar::<i32, _>(&query_view)
        .await
        .map_err(|_| CreateChatError::DatabaseError)? as u64;

    let mut members = view.members().to_vec();
    members.push(user_id);
    // The creator may also be listed: a duplicate would break the (conversation, user) key.
    members.sort_unstable();
    members.dedup();
    let query_view = AddMembersToChatQueryView::new(result, members);
    if let Err(error) = db.execute(query_view).await {
        // Do not leave a conversation without its members behind.
        let _ = db
            .fetch_scalar::<i32, _>(&DeleteChatQueryView::new(result))
            .await;
        return Err(match error {
            ApiLibError::Database(DbError::ForeignKeyViolation(_)) => {
                CreateChatError::UnknownMember
            }
            _ => CreateChatError::DatabaseError,
        });
    }

    Ok(CreateChatResultView::new(result))
}

#[utoipa::path(
    post,
    path = "",
    summary = "Créer une conversation",
    description = "Crée une conversation et y rattache l'appelant ainsi que les participants \
                   listés dans `members`.\n\n\
                   L'appelant est ajouté automatiquement : **ne pas** inclure son propre \
                   identifiant dans `members`. Sinon le rattachement échoue en `500` alors que la \
                   conversation a déjà été créée, sans aucun participant.\n\n\
                   Les identifiants attendus sont ceux des comptes dans Core API. La réponse ne \
                   contient que l'identifiant attribué.",
    responses(
        (
            status = 200,
            description = "Conversation créée. Le corps contient l'identifiant attribué.",
            body = CreateChatResultView,
            example = json!({ "id": 5 })
        ),
        (
            status = 400,
            description = "Malformed JSON body, missing field, `name` empty (direct conversation) or 1 to 150 characters without control characters nor `<` / `>`, or `members` containing an unknown user (`` `members` contains an unknown user.``; no conversation is created).",
            body = String,
            content_type = "text/plain",
            example = json!("`members` contains an unknown user.")
        ),
        (
            status = 401,
            description = "En-tête `Authorization` absent, JWT invalide ou expiré, ou session révoquée.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 500,
            description = "Erreur de base de données, notamment si `members` contient l'appelant ou un doublon. La conversation peut alors avoir été créée sans participant.",
            body = String,
            content_type = "text/plain",
            example = json!("An error occurred while accessing the database.")
        ),
    ),
    request_body(
        content = CreateChatView,
        description = "Titre de la conversation et identifiants Core API de ses participants.",
        example = json!({
            "name": "Service urbanisme",
            "members": [42, 51]
        })
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Chats",
)]
#[post("/")]
pub async fn create_chat(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    view: ValidatedJson<CreateChatView>,
) -> Result<impl Responder, CreateChatError> {
    let view = view.into_inner();
    let result = trigger_create_chat(state, auth_user.id, view).await?;
    Ok(HttpResponse::Ok().json(result))
}
