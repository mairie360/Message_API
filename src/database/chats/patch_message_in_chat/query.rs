use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::patch_message_in_chat::view::PatchMessageQueryView;

pub async fn patch_message_query(
    view: PatchMessageQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<u64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let result = sqlx::query(&view.get_request())
        .bind(view.message_id() as i32)
        .bind(view.content())
        .execute(&pool)
        .await?
        .rows_affected();

    Ok(result)
}
