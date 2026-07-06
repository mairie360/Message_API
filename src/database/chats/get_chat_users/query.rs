use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::get_chat_users::view::GetChatMembersQueryView;

pub async fn get_chat_members_query(
    view: GetChatMembersQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<Vec<i32>, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let users = sqlx::query_scalar::<_, i32>(&view.get_request())
        .bind(view.chat_id() as i32)
        .fetch_all(&pool)
        .await?;

    Ok(users)
}
