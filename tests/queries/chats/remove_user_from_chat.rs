use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_users_to_chat::view::AddMembersToChatQueryView, create_chat::view::CreateChatQueryView,
    remove_user_from_chat::view::RemoveMemberFromChatQueryView,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_remove_user_from_chat_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = AddMembersToChatQueryView::new(chat_id, vec![1]);
    assert!(db.execute(view).await.is_ok());

    let view = RemoveMemberFromChatQueryView::new(chat_id, 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(result.is_ok(), "should succeed, got: {:?}", result);
}

#[tokio::test]
#[serial]
async fn test_remove_user_from_chat_success_unknow_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = AddMembersToChatQueryView::new(999, vec![1]);
    let result = db.execute(view).await;

    assert!(result.is_err(), "should return an error, got: {:?}", result);
}

#[tokio::test]
#[serial]
async fn test_remove_user_from_chat_success_unknow_user() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = RemoveMemberFromChatQueryView::new(chat_id, 999);
    let result = db.fetch_scalar::<i32, _>(&view).await;

    // Aucune ligne supprimée : la lib remonte une erreur `NotFound`.
    assert!(result.is_err(), "should return an error, got: {:?}", result);
}

#[tokio::test]
#[serial]
async fn test_remove_user_from_chat_success_unknow_user_and_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = RemoveMemberFromChatQueryView::new(999, 999);
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(result.is_err(), "should return an error, got: {:?}", result);
}
