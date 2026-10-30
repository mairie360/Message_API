use crate::common::{exclude_member, get_fixture_db, get_smart_db, plain_user};
use message_api::database::chats::{
    add_users_to_chat::view::AddMembersToChatQueryView, create_chat::view::CreateChatQueryView,
    get_chat_users::view::GetChatMembersQueryView,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_get_chat_members_success() {
    let db = get_smart_db().await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap();

    let view = AddMembersToChatQueryView::new(chat_id as u64, vec![1]);
    assert!(db.execute(view).await.is_ok());

    let view = GetChatMembersQueryView::new(chat_id as u64);
    let result = db.fetch_all::<i32, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}

#[tokio::test]
#[serial]
async fn test_get_chat_members_success_unknow_chat() {
    let db = get_smart_db().await;

    let view = GetChatMembersQueryView::new(999);
    let result = db.fetch_all::<i32, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
#[serial]
async fn test_get_chat_members_skips_excluded_members() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let kept = plain_user(&fixtures).await;
    let excluded = plain_user(&fixtures).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Excluded", None))
        .await
        .unwrap() as u64;
    db.execute(AddMembersToChatQueryView::new(
        chat_id,
        vec![kept, excluded],
    ))
    .await
    .unwrap();
    exclude_member(&fixtures, chat_id, excluded).await;

    let members = db
        .fetch_all::<i32, _>(&GetChatMembersQueryView::new(chat_id))
        .await
        .unwrap();
    assert_eq!(members, vec![kept as i32]);
}
