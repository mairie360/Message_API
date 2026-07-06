use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_message_to_chat::{query::post_message_in_chat_query, view::PostMessageInChatQueryView},
    create_chat::{query::create_chat_query, view::CreateChatQueryView},
    delete_message_from_chat::{query::delete_message_query, view::DeleteMessageQueryView},
};

#[sqlx::test]
async fn test_delete_message_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());

    let view = PostMessageInChatQueryView::new(result.unwrap() as u64, 1, "Test Message");
    let result = post_message_in_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());

    let view = DeleteMessageQueryView::new(result.unwrap() as u64);
    let result = delete_message_query(view, pool).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[sqlx::test]
async fn test_delete_message_unknown_message() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = DeleteMessageQueryView::new(999);
    let result = delete_message_query(view, pool).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}
