use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_users_to_chat::view::AddMembersToChatQueryView,
    create_chat::view::CreateChatQueryView,
    get_chats::view::{GetChatsQueryResultView, GetChatsQueryView},
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_get_chats_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    for _ in 0..2 {
        let view = CreateChatQueryView::new("Test Chat", None);
        let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap();

        let view = AddMembersToChatQueryView::new(chat_id as u64, vec![1]);
        assert!(db.execute(view).await.is_ok());
    }

    let view = GetChatsQueryView::new(1);
    let result = db.fetch_all::<GetChatsQueryResultView, _>(&view).await;

    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[tokio::test]
#[serial]
async fn test_get_chats_no_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = GetChatsQueryView::new(2);
    let result = db.fetch_all::<GetChatsQueryResultView, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
#[serial]
async fn test_get_chats_unknown_user() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = GetChatsQueryView::new(999);
    let result = db.fetch_all::<GetChatsQueryResultView, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}
