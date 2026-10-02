use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_message_to_chat::view::PostMessageInChatQueryView, create_chat::view::CreateChatQueryView,
    patch_message_in_chat::view::PatchMessageQueryView,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_patch_message_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap();

    let view = PostMessageInChatQueryView::new(chat_id as u64, 1, "Test Message");
    let message_id = db.fetch_scalar::<i64, _>(&view).await.unwrap();

    let view = PatchMessageQueryView::new(chat_id as u64, message_id as u64, "Updated Content");
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), message_id);

    // The same message through another chat is not found (MAIR-422).
    let view = CreateChatQueryView::new("Other Chat", None);
    let other_chat = db.fetch_scalar::<i32, _>(&view).await.unwrap();
    let view = PatchMessageQueryView::new(other_chat as u64, message_id as u64, "Hijacked");
    assert!(db.fetch_scalar::<i64, _>(&view).await.is_err());
}

#[tokio::test]
#[serial]
async fn test_patch_message_unknown_message() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = PatchMessageQueryView::new(1, 999, "Updated Content");
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_err());
}
