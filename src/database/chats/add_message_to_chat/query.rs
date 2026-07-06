use mairie360_api_lib::database::db_interface::DatabaseQueryView;
use mairie360_api_lib::database::errors::DatabaseError;
use sqlx::PgPool;

use crate::database::chats::add_message_to_chat::view::PostMessageInChatQueryView;

pub async fn post_message_in_chat_query(
    view: PostMessageInChatQueryView,
    pool: PgPool, // ou &PgPool selon ta configuration globale
) -> Result<i64, DatabaseError> {
    // 1. On exécute la requête configurée dans ta view
    let message_id = sqlx::query_scalar::<_, i64>(&view.get_request())
        .bind(view.chat_id() as i32)
        .bind(view.sender() as i32)
        .bind(view.message())
        .fetch_one(&pool)
        .await?;

    Ok(message_id)
}
