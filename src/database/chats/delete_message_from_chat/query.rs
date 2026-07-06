use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::delete_message_from_chat::view::DeleteMessageQueryView;

pub async fn delete_message_query(
    view: DeleteMessageQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<u64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let result = sqlx::query(&view.get_request())
        .bind(view.message_id() as i32)
        .execute(&pool)
        .await?
        .rows_affected();

    Ok(result)
}
