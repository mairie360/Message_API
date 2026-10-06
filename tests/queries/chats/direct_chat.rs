//! Direct chats (MAIR-478): one per pair, hidden instead of left, shown again by a message.

use crate::common::{get_smart_db, grant_chat_delete, plain_user};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    access::view::{ChatAccess, ChatAccessQueryView},
    create_chat::view::CreateChatQueryView,
    delete_empty_chat::view::DeleteEmptyChatQueryView,
    delete_right::view::ChatDeleteRightQueryView,
    get_chats::view::{GetChatsQueryResultView, GetChatsQueryView},
    hide_direct_chat::view::HideDirectChatQueryView,
    open_direct_chat::view::{OpenDirectChatQueryResultView, OpenDirectChatQueryView},
    reveal_direct_chat::view::RevealDirectChatQueryView,
};
use serial_test::serial;

async fn open(db: &SmartDatabase, user: u64, contact: u64) -> OpenDirectChatQueryResultView {
    db.fetch_one(&OpenDirectChatQueryView::new(user, contact))
        .await
        .unwrap()
}

/// The chat `chat_id` as listed for `user`, if they see it.
async fn listed(db: &SmartDatabase, user: u64, chat_id: i32) -> Option<GetChatsQueryResultView> {
    db.fetch_all::<GetChatsQueryResultView, _>(&GetChatsQueryView::new(user))
        .await
        .unwrap()
        .into_iter()
        .find(|chat| chat.id == chat_id)
}

#[tokio::test]
#[serial]
async fn test_a_pair_has_a_single_direct_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let (alice, bob) = (plain_user(&db).await, plain_user(&db).await);

    let first = open(&db, alice, bob).await;
    assert!(first.created);
    let again = open(&db, alice, bob).await;
    let from_bob = open(&db, bob, alice).await;
    assert_eq!((again.id, again.created), (first.id, false));
    assert_eq!((from_bob.id, from_bob.created), (first.id, false));

    let access: ChatAccess = db
        .fetch_one(&ChatAccessQueryView::new(first.id as u64, alice))
        .await
        .unwrap();
    assert!(access.is_direct && access.is_creator);
}

#[tokio::test]
#[serial]
async fn test_direct_chat_lists_its_contact_on_both_sides() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let (alice, bob) = (plain_user(&db).await, plain_user(&db).await);
    let chat = open(&db, alice, bob).await.id;

    let for_alice = listed(&db, alice, chat).await.unwrap();
    assert_eq!(for_alice.kind, "direct");
    assert_eq!(for_alice.contact_id, Some(bob as i32));
    assert_eq!(for_alice.title, None);
    // A new chat stays hidden for the contact until a message is posted in it.
    assert!(listed(&db, bob, chat).await.is_none());

    db.execute(RevealDirectChatQueryView::new(chat as u64))
        .await
        .unwrap();
    let for_bob = listed(&db, bob, chat).await.unwrap();
    assert_eq!(for_bob.contact_id, Some(alice as i32));
}

#[tokio::test]
#[serial]
async fn test_hidden_direct_chat_keeps_its_contact_until_revealed() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let (alice, bob) = (plain_user(&db).await, plain_user(&db).await);
    let chat = open(&db, alice, bob).await.id;
    db.execute(RevealDirectChatQueryView::new(chat as u64))
        .await
        .unwrap();

    let hidden = db
        .fetch_scalar::<i32, _>(&HideDirectChatQueryView::new(chat as u64, bob))
        .await
        .unwrap();
    assert_eq!(hidden, bob as i32);
    assert!(listed(&db, bob, chat).await.is_none());
    // Alice still sees Bob as the contact, and no duplicate is created for the pair.
    assert_eq!(
        listed(&db, alice, chat).await.unwrap().contact_id,
        Some(bob as i32)
    );
    assert_eq!(open(&db, alice, bob).await.id, chat);
    assert!(listed(&db, bob, chat).await.is_none());

    // Hiding twice: nothing to hide.
    assert!(matches!(
        db.fetch_scalar::<i32, _>(&HideDirectChatQueryView::new(chat as u64, bob))
            .await,
        Err(ApiLibError::Database(DbError::NotFound))
    ));

    // Bob opening it himself shows it again.
    assert_eq!(open(&db, bob, alice).await.id, chat);
    assert!(listed(&db, bob, chat).await.is_some());
}

#[tokio::test]
#[serial]
async fn test_direct_chat_hidden_by_both_is_deleted() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let (alice, bob) = (plain_user(&db).await, plain_user(&db).await);
    let chat = open(&db, alice, bob).await.id as u64;

    db.fetch_scalar::<i32, _>(&HideDirectChatQueryView::new(chat, alice))
        .await
        .unwrap();
    db.execute(DeleteEmptyChatQueryView::new(chat))
        .await
        .unwrap();

    let access: ChatAccess = db
        .fetch_one(&ChatAccessQueryView::new(chat, alice))
        .await
        .unwrap();
    assert!(!access.chat_exists);
    // The pair may open a new one.
    assert!(open(&db, alice, bob).await.created);
}

#[tokio::test]
#[serial]
async fn test_direct_chat_with_an_unknown_user_is_refused() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let alice = plain_user(&db).await;

    let result = db
        .fetch_one::<OpenDirectChatQueryResultView, _>(&OpenDirectChatQueryView::new(
            alice,
            i32::MAX as u64,
        ))
        .await;
    assert!(matches!(
        result,
        Err(ApiLibError::Database(DbError::ForeignKeyViolation(_)))
    ));
}

#[tokio::test]
#[serial]
async fn test_reveal_does_not_touch_a_group_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let member = plain_user(&db).await;
    let chat = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::with_members(
            "Service urbanisme",
            None,
            Some(member),
            &[member],
        ))
        .await
        .unwrap();
    crate::common::exclude_member(&db, chat as u64, member).await;

    db.execute(RevealDirectChatQueryView::new(chat as u64))
        .await
        .unwrap();
    assert!(listed(&db, member, chat).await.is_none());
}

#[tokio::test]
#[serial]
async fn test_group_chats_are_listed_as_group_without_contact() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let (creator, member) = (plain_user(&db).await, plain_user(&db).await);
    let chat = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::with_members(
            "",
            None,
            Some(creator),
            &[creator, member],
        ))
        .await
        .unwrap();

    let listed = listed(&db, creator, chat).await.unwrap();
    assert_eq!((listed.kind.as_str(), listed.contact_id), ("group", None));
}

#[tokio::test]
#[serial]
async fn test_delete_right_comes_from_check_access() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let (granted, member) = (plain_user(&db).await, plain_user(&db).await);
    let chat = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::with_members(
            "Astreinte",
            None,
            Some(member),
            &[member],
        ))
        .await
        .unwrap() as u64;

    let right = |user| ChatDeleteRightQueryView::new(chat, user);
    assert!(!db.fetch_scalar::<bool, _>(&right(member)).await.unwrap());
    grant_chat_delete(&db, chat, granted).await;
    assert!(db.fetch_scalar::<bool, _>(&right(granted)).await.unwrap());
    // User 1 is the Admin created by liquibase: `delete_all` on conversations.
    assert!(db.fetch_scalar::<bool, _>(&right(1)).await.unwrap());
}
