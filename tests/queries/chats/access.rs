use crate::common::{get_smart_db, plain_user};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    access::view::{ChatAccess, ChatAccessQueryView},
    add_message_to_chat::view::PostMessageInChatQueryView,
    add_users_to_chat::view::AddMembersToChatQueryView,
    create_chat::view::CreateChatQueryView,
    delete_empty_chat::view::DeleteEmptyChatQueryView,
    message_owner::view::MessageOwnerQueryView,
    remove_user_from_chat::view::RemoveMemberFromChatQueryView,
};
use serial_test::serial;

async fn chat_with(db: &SmartDatabase, members: Vec<u64>) -> u64 {
    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Access", None))
        .await
        .unwrap() as u64;
    db.execute(AddMembersToChatQueryView::new(chat_id, members))
        .await
        .unwrap();
    chat_id
}

async fn access(db: &SmartDatabase, chat_id: u64, user_id: u64) -> ChatAccess {
    db.fetch_one(&ChatAccessQueryView::new(chat_id, user_id))
        .await
        .unwrap()
}

#[tokio::test]
#[serial]
async fn test_chat_access_tells_members_outsiders_and_admins_apart() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let member = plain_user(&db).await;
    let outsider = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![member]).await;

    let member_access = access(&db, chat_id, member).await;
    assert!(member_access.chat_exists && member_access.is_member && !member_access.is_admin);
    // A chat created without `created_by` has no creator.
    assert!(!member_access.is_creator);

    let outsider_access = access(&db, chat_id, outsider).await;
    assert!(outsider_access.chat_exists && !outsider_access.is_member);
    assert!(!outsider_access.is_admin);

    // User 1 is the Admin created by liquibase.
    let admin_access = access(&db, chat_id, 1).await;
    assert!(admin_access.is_admin && !admin_access.is_member);
}

#[tokio::test]
#[serial]
async fn test_chat_access_on_an_unknown_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let user = plain_user(&db).await;

    let unknown = access(&db, 999_999, user).await;
    assert!(!unknown.chat_exists && !unknown.is_member);
}

#[tokio::test]
#[serial]
async fn test_message_owner_is_scoped_to_the_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let author = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![author]).await;
    let other_chat = chat_with(&db, vec![author]).await;
    let message_id = db
        .fetch_scalar::<i64, _>(&PostMessageInChatQueryView::new(chat_id, author, "Bonjour"))
        .await
        .unwrap() as u64;

    let owner: i32 = db
        .fetch_scalar(&MessageOwnerQueryView::new(chat_id, message_id))
        .await
        .unwrap();
    assert_eq!(owner as u64, author);

    let result = db
        .fetch_scalar::<i32, _>(&MessageOwnerQueryView::new(other_chat, message_id))
        .await;
    assert!(
        matches!(result, Err(ApiLibError::Database(DbError::NotFound))),
        "a message of another chat must not be found, got {result:?}"
    );
}

#[tokio::test]
#[serial]
async fn test_chat_is_deleted_with_its_last_member() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let first = plain_user(&db).await;
    let second = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![first, second]).await;

    db.fetch_scalar::<i32, _>(&RemoveMemberFromChatQueryView::new(chat_id, first))
        .await
        .unwrap();
    db.execute(DeleteEmptyChatQueryView::new(chat_id))
        .await
        .unwrap();
    assert!(access(&db, chat_id, second).await.chat_exists);

    db.fetch_scalar::<i32, _>(&RemoveMemberFromChatQueryView::new(chat_id, second))
        .await
        .unwrap();
    db.execute(DeleteEmptyChatQueryView::new(chat_id))
        .await
        .unwrap();
    assert!(!access(&db, chat_id, second).await.chat_exists);
}

#[tokio::test]
#[serial]
async fn test_chat_access_tells_the_creator_apart() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let creator = plain_user(&db).await;
    let member = plain_user(&db).await;
    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::with_members(
            "Creator",
            None,
            Some(creator),
            &[creator, member],
        ))
        .await
        .unwrap() as u64;

    assert!(access(&db, chat_id, creator).await.is_creator);
    assert!(!access(&db, chat_id, member).await.is_creator);

    // A creator who left is still recorded, but no longer a member.
    db.fetch_scalar::<i32, _>(&RemoveMemberFromChatQueryView::new(chat_id, creator))
        .await
        .unwrap();
    let left = access(&db, chat_id, creator).await;
    assert!(left.is_creator && !left.is_member);
}

#[tokio::test]
#[serial]
async fn test_locking_access_answers_like_the_plain_one() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let creator = plain_user(&db).await;
    let member = plain_user(&db).await;
    let outsider = plain_user(&db).await;
    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::with_members(
            "Locking",
            None,
            Some(creator),
            &[creator, member],
        ))
        .await
        .unwrap() as u64;

    for (chat, user) in [
        (chat_id, creator),
        (chat_id, member),
        (chat_id, outsider),
        (chat_id, 1),
        (999_999, member),
    ] {
        let mut tx = db.begin().await.unwrap();
        let locked: ChatAccess = tx
            .fetch_one(&ChatAccessQueryView::locking(chat, user))
            .await
            .unwrap();
        tx.rollback().await.unwrap();
        let plain = access(&db, chat, user).await;
        assert_eq!(
            (
                locked.chat_exists,
                locked.is_member,
                locked.is_creator,
                locked.is_admin
            ),
            (
                plain.chat_exists,
                plain.is_member,
                plain.is_creator,
                plain.is_admin
            ),
            "chat {chat} user {user}"
        );
    }
}

/// MAIR-420: a write checks access and writes in one transaction; until it ends, the member it
/// checked cannot be removed (no post from a member already gone).
#[tokio::test]
#[serial]
async fn test_locking_access_holds_the_membership_until_the_write_ends() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let member = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![member]).await;

    let mut tx = db.begin().await.unwrap();
    let checked: ChatAccess = tx
        .fetch_one(&ChatAccessQueryView::locking(chat_id, member))
        .await
        .unwrap();
    assert!(checked.is_member);

    let remover = db.clone();
    let removal = tokio::spawn(async move {
        remover
            .fetch_scalar::<i32, _>(&RemoveMemberFromChatQueryView::new(chat_id, member))
            .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert!(
        !removal.is_finished(),
        "the removal must wait for the transaction that checked the membership"
    );

    tx.fetch_scalar::<i64, _>(&PostMessageInChatQueryView::new(
        chat_id,
        member,
        "Juste à temps",
    ))
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert!(removal.await.unwrap().is_ok());
    assert!(!access(&db, chat_id, member).await.is_member);
}
