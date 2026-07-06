use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::get_chats::view::{GetChatsQueryResultView, GetChatsQueryView};

pub async fn get_chats_query(
    view: GetChatsQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<Vec<GetChatsQueryResultView>, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let chats = sqlx::query_as::<_, GetChatsQueryResultView>(&view.get_request())
        .bind(view.user_id() as i32)
        .fetch_all(&pool)
        .await?;

    Ok(chats)
}
