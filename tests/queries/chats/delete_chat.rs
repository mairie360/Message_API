use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    create_chat::view::CreateChatQueryView, delete_chat::view::DeleteChatQueryView,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_delete_chat_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap();

    let view = DeleteChatQueryView::new(chat_id as u64);
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), chat_id);
}

#[tokio::test]
#[serial]
async fn test_delete_unknow_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = DeleteChatQueryView::new(999);
    let result = db.fetch_scalar::<i32, _>(&view).await;

    // Aucune ligne supprimée : la lib remonte une erreur `NotFound`.
    assert!(result.is_err());
}
