use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::get_chat::view::{GetChatQueryView, Message};

pub async fn get_chat_query(
    view: GetChatQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<Vec<Message>, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let chats = sqlx::query_as::<_, Message>(&view.get_request())
        .bind(view.chat_id() as i32)
        .fetch_all(&pool)
        .await?;

    Ok(chats)
}
