use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::reset_unread_count::view::ResetUnreadCountQueryView;

pub async fn reset_unread_count_query(
    view: ResetUnreadCountQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<u64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let result = sqlx::query(&view.get_request())
        .bind(view.chat_id() as i32)
        .bind(view.user_id() as i32)
        .execute(&pool)
        .await?
        .rows_affected();

    Ok(result)
}
