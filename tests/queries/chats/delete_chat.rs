use crate::common::{get_smart_db, moderation_log_count};
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

    let view = DeleteChatQueryView::new(chat_id as u64, 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), chat_id);
    assert_eq!(
        moderation_log_count(&db, chat_id as u64, "DELETE_CONVERSATION", 1).await,
        1
    );
}

#[tokio::test]
#[serial]
async fn test_delete_unknow_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = DeleteChatQueryView::new(999, 1);
    let result = db.fetch_scalar::<i32, _>(&view).await;

    // No row deleted: the lib reports `NotFound`, and nothing is logged.
    assert!(result.is_err());
    assert_eq!(
        moderation_log_count(&db, 999, "DELETE_CONVERSATION", 1).await,
        0
    );
}
