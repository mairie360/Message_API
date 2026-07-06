use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::delete_chat::view::DeleteChatQueryView;

pub async fn delete_chat_query(
    view: DeleteChatQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<u64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let result = sqlx::query(&view.get_request())
        .bind(view.chat_id() as i32)
        .execute(&pool)
        .await?;

    Ok(result.rows_affected() as u64)
}
