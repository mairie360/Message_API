use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_users_to_chat::view::AddMembersToChatQueryView, create_chat::view::CreateChatQueryView,
    reset_unread_count::view::ResetUnreadCountQueryView,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_reset_unread_count_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    let view = AddMembersToChatQueryView::new(chat_id, vec![1]);
    assert!(db.execute(view).await.is_ok());

    let view = ResetUnreadCountQueryView::new(chat_id, 1);
    let result = db.execute(view).await;

    assert!(result.is_ok(), "should succeed, got: {result:?}");
}

#[tokio::test]
#[serial]
async fn test_reset_unread_count_no_matching_counter() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    // `UPDATE` sans clause `RETURNING` : aucune ligne concernée n'est pas une
    // erreur, l'appel est un no-op silencieux.
    let view = ResetUnreadCountQueryView::new(999, 999);
    let result = db.execute(view).await;

    assert!(result.is_ok(), "should be a silent no-op, got: {result:?}");
}
