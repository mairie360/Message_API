use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_message_to_chat::{query::post_message_in_chat_query, view::PostMessageInChatQueryView},
    create_chat::{query::create_chat_query, view::CreateChatQueryView},
    get_chat::{query::get_chat_query, view::GetChatQueryView},
};

#[sqlx::test]
async fn test_get_chat_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());
    let chat_id = result.unwrap() as u64;

    let view = PostMessageInChatQueryView::new(chat_id, 1, "Test Message");
    let _ = post_message_in_chat_query(view, pool.clone()).await;

    let view = PostMessageInChatQueryView::new(chat_id, 1, "Test Message");
    let _ = post_message_in_chat_query(view, pool.clone()).await;

    let view = PostMessageInChatQueryView::new(chat_id, 1, "Test Message");
    let _ = post_message_in_chat_query(view, pool.clone()).await;

    let view = GetChatQueryView::new(chat_id);
    let result = get_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());
    let messages = result.unwrap();
    assert_eq!(messages.len(), 3);
}

#[sqlx::test]
async fn test_get_chat_unknown_chat() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = GetChatQueryView::new(999);
    let result = get_chat_query(view, pool).await;

    assert!(result.is_err());
}
