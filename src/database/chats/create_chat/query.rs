use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::create_chat::view::CreateChatQueryView;

pub async fn create_chat_query(
    view: CreateChatQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<i32, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let chat = sqlx::query_scalar::<_, i32>(&view.get_request())
        .bind(view.title())
        .bind(view.group_id())
        .fetch_one(&pool)
        .await?;

    Ok(chat)
}
