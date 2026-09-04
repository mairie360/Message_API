use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_message_to_chat::view::PostMessageInChatQueryView, create_chat::view::CreateChatQueryView,
    delete_message_from_chat::view::DeleteMessageQueryView,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_delete_message_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap();

    let view = PostMessageInChatQueryView::new(chat_id as u64, 1, "Test Message");
    let message_id = db.fetch_scalar::<i64, _>(&view).await.unwrap();

    let view = DeleteMessageQueryView::new(message_id as u64);
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), message_id);
}

#[tokio::test]
#[serial]
async fn test_delete_message_unknown_message() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = DeleteMessageQueryView::new(999);
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_err());
}
