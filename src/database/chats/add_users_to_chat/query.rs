use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::add_users_to_chat::view::AddMembersToChatQueryView;

pub async fn add_members_to_chat_query(
    view: AddMembersToChatQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<(), DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    sqlx::query(&view.get_request())
        .bind(view.chat_id() as i32)
        .bind(view.user_id().to_vec())
        .execute(&pool)
        .await?;

    Ok(())
}
