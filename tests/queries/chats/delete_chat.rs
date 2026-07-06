use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    create_chat::{query::create_chat_query, view::CreateChatQueryView},
    delete_chat::{query::delete_chat_query, view::DeleteChatQueryView},
};

#[sqlx::test]
async fn test_delete_chat_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool.clone()).await;

    let view = DeleteChatQueryView::new(result.unwrap() as u64);
    let result = delete_chat_query(view, pool).await;

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);
}

#[sqlx::test]
async fn test_delete_unknow_chat() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = DeleteChatQueryView::new(999);
    let result = delete_chat_query(view, pool).await;

    assert!(result.is_ok());
    assert!(result.unwrap() == 0);
}
