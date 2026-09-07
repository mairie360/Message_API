use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_message_to_chat::view::PostMessageInChatQueryView, create_chat::view::CreateChatQueryView,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_add_message_to_chat_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await;
    assert!(chat_id.is_ok());

    let view = PostMessageInChatQueryView::new(chat_id.unwrap() as u64, 1, "Test Message");
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_ok());
    assert!(result.unwrap() != 0);
}

#[tokio::test]
#[serial]
async fn test_add_message_to_chat_unknown_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = PostMessageInChatQueryView::new(999, 1, "Test Message");
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_add_message_to_chat_unknown_sender() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await;
    assert!(chat_id.is_ok());

    let view = PostMessageInChatQueryView::new(chat_id.unwrap() as u64, 999, "Test Message");
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_add_message_to_chat_unknown_sender_and_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = PostMessageInChatQueryView::new(999, 999, "Test Message");
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_err());
}
