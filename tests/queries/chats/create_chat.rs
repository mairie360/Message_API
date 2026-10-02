use crate::common::{get_smart_db, plain_user};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    access::view::{ChatAccess, ChatAccessQueryView},
    create_chat::view::CreateChatQueryView,
    get_chat_users::view::GetChatMembersQueryView,
};
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

#[tokio::test]
#[serial]
async fn test_create_chat_with_members_records_the_creator() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let creator = plain_user(&db).await;
    let member = plain_user(&db).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::with_members(
            "Service urbanisme",
            None,
            Some(creator),
            &[creator, member],
        ))
        .await
        .unwrap() as u64;

    let mut members = db
        .fetch_all::<i32, _>(&GetChatMembersQueryView::new(chat_id))
        .await
        .unwrap();
    members.sort_unstable();
    let mut expected = vec![creator as i32, member as i32];
    expected.sort_unstable();
    assert_eq!(members, expected);

    let creator_access: ChatAccess = db
        .fetch_one(&ChatAccessQueryView::new(chat_id, creator))
        .await
        .unwrap();
    assert!(creator_access.is_creator && creator_access.is_member);
    let member_access: ChatAccess = db
        .fetch_one(&ChatAccessQueryView::new(chat_id, member))
        .await
        .unwrap();
    assert!(!member_access.is_creator && member_access.is_member);
}

#[tokio::test]
#[serial]
async fn test_create_chat_with_an_unknown_member_creates_nothing() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let creator = plain_user(&db).await;

    let view =
        CreateChatQueryView::with_members("Atomic", None, Some(creator), &[creator, 999_999_999]);
    let result = db.fetch_scalar::<i32, _>(&view).await;
    assert!(
        matches!(
            result,
            Err(ApiLibError::Database(DbError::ForeignKeyViolation(_)))
        ),
        "an unknown member must fail the statement, got {result:?}"
    );

    // The chat insert was rolled back with the members: the next chat id was never used by a
    // chat of this creator.
    let next_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::with_members(
            "After",
            None,
            Some(creator),
            &[creator],
        ))
        .await
        .unwrap() as u64;
    let previous: ChatAccess = db
        .fetch_one(&ChatAccessQueryView::new(next_id - 1, creator))
        .await
        .unwrap();
    assert!(!(previous.chat_exists && previous.is_creator));
}
