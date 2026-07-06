use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_users_to_chat::{query::add_members_to_chat_query, view::AddMembersToChatQueryView},
    create_chat::{query::create_chat_query, view::CreateChatQueryView},
    get_chats::{query::get_chats_query, view::GetChatsQueryView},
};

#[sqlx::test]
async fn test_get_chats_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());

    let view = AddMembersToChatQueryView::new(result.unwrap() as u64, vec![1]);
    let result = add_members_to_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());

    let view = AddMembersToChatQueryView::new(result.unwrap() as u64, vec![1]);
    let result = add_members_to_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());

    let view = GetChatsQueryView::new(1);
    let result = get_chats_query(view, pool).await;

    assert!(result.is_ok());
    assert!(result.unwrap().len() > 0);
}

#[sqlx::test]
async fn test_get_chats_no_chat() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = GetChatsQueryView::new(2);
    let result = get_chats_query(view, pool).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[sqlx::test]
async fn test_get_chats_unknown_user() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = GetChatsQueryView::new(999);
    let result = get_chats_query(view, pool).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}
