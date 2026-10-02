use crate::common::get_smart_db;
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
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

#[tokio::test]
#[serial]
async fn test_add_message_cannot_quote_a_message_of_another_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Here", None))
        .await
        .unwrap() as u64;
    let other_chat = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Elsewhere", None))
        .await
        .unwrap() as u64;
    let foreign = db
        .fetch_scalar::<i64, _>(&PostMessageInChatQueryView::new(other_chat, 1, "Elsewhere"))
        .await
        .unwrap();

    let view = PostMessageInChatQueryView::replying_to(chat_id, 1, "Reply", Some(foreign as u64));
    let result = db.fetch_scalar::<i64, _>(&view).await;
    assert!(
        matches!(
            &result,
            Err(ApiLibError::Database(DbError::ForeignKeyViolation(message)))
                if message.contains("reply_to")
        ),
        "a citation of another chat must break fk_messages_reply_to, got {result:?}"
    );

    let unknown = PostMessageInChatQueryView::replying_to(chat_id, 1, "Reply", Some(999_999_999));
    assert!(db.fetch_scalar::<i64, _>(&unknown).await.is_err());
}
