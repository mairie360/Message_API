use crate::common::get_smart_db;
use message_api::database::chats::{
    add_users_to_chat::view::AddMembersToChatQueryView, create_chat::view::CreateChatQueryView,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_add_user_to_chat_success() {
    let db = get_smart_db().await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await;
    assert!(chat_id.is_ok());

    let view = AddMembersToChatQueryView::new(chat_id.unwrap() as u64, vec![1]);
    let result = db.execute(view).await;

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_add_user_to_chat_success_unknow_chat() {
    let db = get_smart_db().await;

    let view = AddMembersToChatQueryView::new(999, vec![1]);
    let result = db.execute(view).await;

    assert!(result.is_err());
}
