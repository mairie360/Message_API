use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_users_to_chat::{query::add_members_to_chat_query, view::AddMembersToChatQueryView},
    create_chat::{query::create_chat_query, view::CreateChatQueryView},
    remove_user_from_chat::{
        query::remove_member_from_chat_query, view::RemoveMemberFromChatQueryView,
    },
};

#[sqlx::test]
async fn test_remove_user_from_chat_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "should return an error, return: {:?}",
        result
    );
    let chat_id = result.unwrap() as u64;

    let view = AddMembersToChatQueryView::new(chat_id, vec![1]);
    let result = add_members_to_chat_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "should return an error, return: {:?}",
        result
    );

    let view = RemoveMemberFromChatQueryView::new(chat_id, 1);
    let result = remove_member_from_chat_query(view, pool).await;

    assert!(
        result.is_ok(),
        "should return an error, return: {:?}",
        result
    );
    assert_eq!(result.unwrap(), 1);
}

#[sqlx::test]
async fn test_remove_user_from_chat_success_unknow_chat() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = AddMembersToChatQueryView::new(999, vec![1]);
    let result = add_members_to_chat_query(view, pool).await;

    assert!(
        result.is_err(),
        "should return an error, return: {:?}",
        result
    );
}

#[sqlx::test]
async fn test_remove_user_from_chat_success_unknow_user() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool.clone()).await;

    assert!(
        result.is_ok(),
        "should return an error, return: {:?}",
        result
    );
    let chat_id = result.unwrap() as u64;

    let view = RemoveMemberFromChatQueryView::new(chat_id, 999);
    let result = remove_member_from_chat_query(view, pool).await;

    assert!(
        result.is_ok(),
        "should return an error, return: {:?}",
        result
    );
    let result = result.unwrap();
    assert_eq!(result, 0, "should return 0, result: {:?}", result);
}

#[sqlx::test]
async fn test_remove_user_from_chat_success_unknow_user_and_chat() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = RemoveMemberFromChatQueryView::new(999, 999);
    let result = remove_member_from_chat_query(view, pool).await;

    assert!(result.is_ok(), "should return 0, return: {:?}", result);
    let result = result.unwrap();
    assert_eq!(result, 0, "should return 0, result: {:?}", result);
}
