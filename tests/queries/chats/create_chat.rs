use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::create_chat::view::CreateChatQueryView;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_chat_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(result.is_ok());
    assert!(result.unwrap() != 0);
}

#[tokio::test]
#[serial]
async fn test_create_chat_by_group() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat by Group", Some(1));

    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(result.is_ok());
    assert!(result.unwrap() != 0);
}

#[tokio::test]
#[serial]
async fn test_create_chat_by_group_unknown_group() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat by Group", Some(999));

    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(result.is_err());
}
