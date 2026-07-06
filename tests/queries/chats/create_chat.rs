use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::create_chat::{
    query::create_chat_query, view::CreateChatQueryView,
};

#[sqlx::test]
async fn test_create_chat_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool).await;

    assert!(result.is_ok());
    assert!(result.unwrap() != 0);
}

#[sqlx::test]
async fn test_create_chat_by_group() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat by Group", Some(1));

    let result = create_chat_query(view, pool).await;

    assert!(result.is_ok());
    assert!(result.unwrap() != 0);
}

#[sqlx::test]
async fn test_create_chat_by_group_unknown_group() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat by Group", Some(999));

    let result = create_chat_query(view, pool).await;

    assert!(result.is_err());
}
