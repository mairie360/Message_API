use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_message_to_chat::view::PostMessageInChatQueryView,
    create_chat::view::CreateChatQueryView,
    get_chat::view::{GetChatQueryView, Message},
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_get_chat_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    for _ in 0..3 {
        let view = PostMessageInChatQueryView::new(chat_id, 1, "Test Message");
        let _ = db.fetch_scalar::<i64, _>(&view).await;
    }

    let view = GetChatQueryView::new(chat_id);
    let result = db.fetch_all::<Message, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 3);
}

#[tokio::test]
#[serial]
async fn test_get_chat_unknown_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = GetChatQueryView::new(999);
    let result = db.fetch_all::<Message, _>(&view).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
