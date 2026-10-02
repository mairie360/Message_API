use crate::common::{get_smart_db, moderation_log_count, plain_user};
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
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = PostMessageInChatQueryView::new(chat_id, 1, "Test Message");
    let message_id = db.fetch_scalar::<i64, _>(&view).await.unwrap();

    let view = DeleteMessageQueryView::new(chat_id, message_id as u64, 1);
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), message_id);
    // The author deleting their own message is not moderation.
    assert_eq!(
        moderation_log_count(&db, chat_id, "DELETE_MESSAGE", 1).await,
        0
    );
}

#[tokio::test]
#[serial]
async fn test_delete_message_unknown_message() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = DeleteMessageQueryView::new(1, 999, 1);
    let result = db.fetch_scalar::<i64, _>(&view).await;

    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_delete_message_is_scoped_to_its_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Owner chat", None))
        .await
        .unwrap() as u64;
    let other_chat = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Other chat", None))
        .await
        .unwrap() as u64;
    let message_id = db
        .fetch_scalar::<i64, _>(&PostMessageInChatQueryView::new(chat_id, 1, "Kept"))
        .await
        .unwrap() as u64;

    let result = db
        .fetch_scalar::<i64, _>(&DeleteMessageQueryView::new(other_chat, message_id, 1))
        .await;
    assert!(
        result.is_err(),
        "a message of another chat must be left alone"
    );
}

#[tokio::test]
#[serial]
async fn test_moderating_a_message_is_logged() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let author = plain_user(&db).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Moderated", None))
        .await
        .unwrap() as u64;
    let message_id = db
        .fetch_scalar::<i64, _>(&PostMessageInChatQueryView::new(chat_id, author, "Spam"))
        .await
        .unwrap() as u64;

    // User 1 (the seeded Admin) deletes the message of `author`.
    db.fetch_scalar::<i64, _>(&DeleteMessageQueryView::new(chat_id, message_id, 1))
        .await
        .unwrap();
    assert_eq!(
        moderation_log_count(&db, chat_id, "DELETE_MESSAGE", 1).await,
        1
    );
}
