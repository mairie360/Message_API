//! MAIR-524: change tracking of the messages (Database 3.1.0). Reads since a cursor, soft delete
//! (tombstones) and what every other read does with tombstones.

use crate::common::{
    force_updated_at, get_fixture_db, get_smart_db, plain_user, unread_count, RawQuery,
};
use chrono::{DateTime, Duration, Utc};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::smart_db::SmartDatabase;
use message_api::database::chats::{
    acknowledge_read::view::AcknowledgeReadQueryView,
    add_message_to_chat::view::PostMessageInChatQueryView,
    add_users_to_chat::view::AddMembersToChatQueryView,
    create_chat::view::CreateChatQueryView,
    delete_message_from_chat::view::{DeleteMessageQueryView, LockChatMessagesQueryView},
    get_chat::view::{
        ChatHeader, GetChatChangesQueryView, GetChatHeaderQueryView, GetChatQueryView, Message,
    },
    get_chats::view::{GetChatsQueryResultView, GetChatsQueryView},
    patch_message_in_chat::view::PatchMessageQueryView,
};
use serial_test::serial;

async fn chat_with(db: &SmartDatabase, title: &str, members: Vec<u64>) -> u64 {
    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new(title, None))
        .await
        .unwrap() as u64;
    db.execute(AddMembersToChatQueryView::new(chat_id, members))
        .await
        .unwrap();
    chat_id
}

async fn post(db: &SmartDatabase, chat_id: u64, sender: u64, content: &str) -> i64 {
    db.fetch_scalar::<i64, _>(&PostMessageInChatQueryView::new(chat_id, sender, content))
        .await
        .unwrap()
}

async fn changes(db: &SmartDatabase, chat_id: u64, since: DateTime<Utc>, id: u64) -> Vec<Message> {
    db.fetch_all::<Message, _>(&GetChatChangesQueryView::new(chat_id, since, id, 100))
        .await
        .unwrap()
}

/// Deletes as the endpoint does: conversation lock, then the soft delete, in one transaction.
async fn delete(
    db: &SmartDatabase,
    chat_id: u64,
    message_id: i64,
    by: u64,
) -> Result<i64, ApiLibError> {
    let mut tx = db.begin().await.unwrap();
    tx.execute(&LockChatMessagesQueryView::new(chat_id))
        .await
        .unwrap();
    let result = tx
        .fetch_scalar::<i64, _>(&DeleteMessageQueryView::new(chat_id, message_id as u64, by))
        .await;
    tx.commit().await.unwrap();
    result
}

fn ids(rows: &[Message]) -> Vec<i64> {
    rows.iter().map(|m| m.id).collect()
}

#[tokio::test]
#[serial]
async fn test_mair_524_since_returns_created_edited_and_deleted_messages_in_change_order() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let author = plain_user(&fixtures).await;
    let chat_id = chat_with(&db, "Since", vec![author]).await;

    let before = Utc::now() - Duration::seconds(5);
    let first = post(&db, chat_id, author, "Premier").await;
    let second = post(&db, chat_id, author, "Deuxième").await;
    let third = post(&db, chat_id, author, "Troisième").await;

    let rows = changes(&db, chat_id, before, 0).await;
    assert_eq!(ids(&rows), vec![first, second, third]);
    assert!(rows.iter().all(|m| !m.deleted && m.edited_at.is_none()));
    let cursor = rows.last().unwrap();
    let (since, since_id) = (cursor.updated_at, cursor.id as u64);
    assert!(changes(&db, chat_id, since, since_id).await.is_empty());

    // An edit and a deletion move the first two messages after the cursor, in that order.
    db.fetch_scalar::<i64, _>(&PatchMessageQueryView::new(
        chat_id,
        first as u64,
        "Corrigé",
    ))
    .await
    .unwrap();
    delete(&db, chat_id, second, author).await.unwrap();

    let rows = changes(&db, chat_id, since, since_id).await;
    assert_eq!(ids(&rows), vec![first, second]);
    assert_eq!(rows[0].content, "Corrigé");
    assert!(rows[0].edited_at.is_some() && !rows[0].deleted);
    assert!(rows[1].deleted, "the deletion comes as a tombstone");
    assert_eq!(rows[1].content, "", "a tombstone keeps no content");
    assert!(rows[0].updated_at < rows[1].updated_at);
}

#[tokio::test]
#[serial]
async fn test_mair_524_since_pages_through_equal_timestamps_with_the_id() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let author = plain_user(&fixtures).await;
    let chat_id = chat_with(&db, "Equal timestamps", vec![author]).await;

    let mut all = Vec::new();
    for n in 0..5 {
        all.push(post(&db, chat_id, author, &format!("message {n}")).await);
    }
    // Five changes at the very same instant: only the id tells them apart.
    let at = Utc::now() + Duration::seconds(1);
    force_updated_at(&fixtures, &all, at).await;

    let page = |since, id| {
        let db = db.clone();
        async move {
            db.fetch_all::<Message, _>(&GetChatChangesQueryView::new(chat_id, since, id, 2))
                .await
        }
    };
    let first = page(at - Duration::seconds(1), 0).await.unwrap();
    assert_eq!(ids(&first), all[0..2]);
    let last = first.last().unwrap();
    let second = page(last.updated_at, last.id as u64).await.unwrap();
    assert_eq!(
        ids(&second),
        all[2..4],
        "no overlap nor gap at equal timestamps"
    );
    let last = second.last().unwrap();
    let third = page(last.updated_at, last.id as u64).await.unwrap();
    assert_eq!(ids(&third), all[4..5]);
    let last = third.last().unwrap();
    assert!(page(last.updated_at, last.id as u64)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
#[serial]
async fn test_mair_524_since_is_scoped_to_its_chat() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let author = plain_user(&fixtures).await;
    let chat_id = chat_with(&db, "Mine", vec![author]).await;
    let other = chat_with(&db, "Other", vec![author]).await;
    let since = Utc::now() - Duration::seconds(5);
    post(&db, other, author, "Ailleurs").await;
    let mine = post(&db, chat_id, author, "Ici").await;

    assert_eq!(ids(&changes(&db, chat_id, since, 0).await), vec![mine]);
}

#[tokio::test]
#[serial]
async fn test_mair_524_since_query_uses_the_change_index() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let author = plain_user(&fixtures).await;
    let chat_id = chat_with(&db, "Explain", vec![author]).await;
    post(&db, chat_id, author, "Index").await;

    let view = GetChatChangesQueryView::new(chat_id, Utc::now() - Duration::hours(1), 0, 51);
    let mut tx = db.begin().await.unwrap();
    // The test table is tiny: forbid the sequential scan a real volume would never pick.
    tx.execute(&RawQuery::new("SET LOCAL enable_seqscan = off", vec![]))
        .await
        .unwrap();
    let plan = tx
        .fetch_one::<serde_json::Value, _>(&RawQuery::explain(&view))
        .await
        .unwrap();
    tx.rollback().await.unwrap();

    let plan = plan.to_string();
    assert!(
        plan.contains("idx_messages_conversation_updated"),
        "the since read must use idx_messages_conversation_updated: {plan}"
    );
    assert!(
        !plan.contains("\"Node Type\":\"Sort\""),
        "the index gives the change order, no sort expected: {plan}"
    );
}

#[tokio::test]
#[serial]
async fn test_mair_524_history_skips_tombstones_and_their_quotes() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let author = plain_user(&fixtures).await;
    let chat_id = chat_with(&db, "Tombstones", vec![author]).await;

    let quoted = post(&db, chat_id, author, "Sera supprimé").await;
    let reply = db
        .fetch_scalar::<i64, _>(&PostMessageInChatQueryView::replying_to(
            chat_id,
            author,
            "Réponse",
            Some(quoted as u64),
        ))
        .await
        .unwrap();
    assert_eq!(delete(&db, chat_id, quoted, author).await.unwrap(), quoted);

    let history = db
        .fetch_all::<Message, _>(&GetChatQueryView::new(chat_id, None, 50))
        .await
        .unwrap();
    assert_eq!(
        ids(&history),
        vec![reply],
        "the tombstone is not in the history"
    );
    let reply_row = &history[0];
    assert_eq!(
        reply_row.reply_to_id, None,
        "the quote of a deleted message is hidden"
    );
    assert_eq!(reply_row.reply_excerpt, None);
    assert!(!reply_row.deleted);

    // The incremental read hides the quote too.
    let rows = changes(&db, chat_id, Utc::now() - Duration::hours(1), 0).await;
    let reply_change = rows.iter().find(|m| m.id == reply).unwrap();
    assert_eq!(reply_change.reply_excerpt, None);
}

#[tokio::test]
#[serial]
async fn test_mair_524_a_tombstone_is_neither_edited_deleted_again_nor_quoted() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let author = plain_user(&fixtures).await;
    let chat_id = chat_with(&db, "Final tombstone", vec![author]).await;
    let message = post(&db, chat_id, author, "Adieu").await;
    delete(&db, chat_id, message, author).await.unwrap();

    let patched = db
        .fetch_scalar::<i64, _>(&PatchMessageQueryView::new(
            chat_id,
            message as u64,
            "Retour",
        ))
        .await;
    assert!(matches!(
        patched,
        Err(ApiLibError::Database(DbError::NotFound))
    ));
    assert!(matches!(
        delete(&db, chat_id, message, author).await,
        Err(ApiLibError::Database(DbError::NotFound))
    ));
    let quoting = db
        .fetch_scalar::<i64, _>(&PostMessageInChatQueryView::replying_to(
            chat_id,
            author,
            "Je cite",
            Some(message as u64),
        ))
        .await;
    assert!(matches!(
        quoting,
        Err(ApiLibError::Database(DbError::NotFound))
    ));
}

#[tokio::test]
#[serial]
async fn test_mair_524_deleting_an_unread_message_lowers_the_unread_counter() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let author = plain_user(&fixtures).await;
    let reader = plain_user(&fixtures).await;
    let late_reader = plain_user(&fixtures).await;
    let chat_id = chat_with(&db, "Unread", vec![author, reader]).await;

    let read = post(&db, chat_id, author, "Lu").await;
    let unread = post(&db, chat_id, author, "Pas lu").await;
    let kept = post(&db, chat_id, author, "Gardé").await;
    db.fetch_scalar::<i32, _>(&AcknowledgeReadQueryView::new(chat_id, reader, read as u64))
        .await
        .unwrap();
    assert_eq!(unread_count(&fixtures, chat_id, reader).await, 2);
    // Joined after the messages: counted none of them.
    db.execute(AddMembersToChatQueryView::new(chat_id, vec![late_reader]))
        .await
        .unwrap();
    let after_join = post(&db, chat_id, author, "Bienvenue").await;
    assert_eq!(unread_count(&fixtures, chat_id, late_reader).await, 1);

    // Deleting a message already read changes nothing, an unread one lowers the counter.
    delete(&db, chat_id, read, author).await.unwrap();
    assert_eq!(unread_count(&fixtures, chat_id, reader).await, 3);
    delete(&db, chat_id, unread, author).await.unwrap();
    assert_eq!(unread_count(&fixtures, chat_id, reader).await, 2);
    assert_eq!(
        unread_count(&fixtures, chat_id, late_reader).await,
        1,
        "a message posted before joining was never counted"
    );
    assert_eq!(unread_count(&fixtures, chat_id, author).await, 0);

    // The recount of an acknowledgement agrees, and the list shows the same number.
    let recount = db
        .fetch_scalar::<i32, _>(&AcknowledgeReadQueryView::new(chat_id, reader, read as u64))
        .await
        .unwrap();
    assert_eq!(recount, 2, "{kept} and {after_join} remain unread");
    let line = db
        .fetch_all::<GetChatsQueryResultView, _>(&GetChatsQueryView::new(reader))
        .await
        .unwrap()
        .into_iter()
        .find(|chat| chat.id as u64 == chat_id)
        .unwrap();
    assert_eq!(line.unread_count, 2);
}

#[tokio::test]
#[serial]
async fn test_mair_524_header_carries_the_latest_change_cursor() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let author = plain_user(&fixtures).await;
    let chat_id = chat_with(&db, "Cursor", vec![author]).await;

    let empty = db
        .fetch_one::<ChatHeader, _>(&GetChatHeaderQueryView::new(chat_id, author))
        .await
        .unwrap();
    assert_eq!(empty.latest_change_id, 0);
    assert_eq!(empty.latest_change_at, empty.chat.last_activity_at);

    let first = post(&db, chat_id, author, "Un").await;
    post(&db, chat_id, author, "Deux").await;
    db.fetch_scalar::<i64, _>(&PatchMessageQueryView::new(chat_id, first as u64, "Un bis"))
        .await
        .unwrap();
    let header = db
        .fetch_one::<ChatHeader, _>(&GetChatHeaderQueryView::new(chat_id, author))
        .await
        .unwrap();
    assert_eq!(
        header.latest_change_id, first,
        "the last edit is the latest change"
    );
    assert_eq!(header.latest_change_at, header.chat.last_activity_at);
    assert!(changes(
        &db,
        chat_id,
        header.latest_change_at,
        header.latest_change_id as u64
    )
    .await
    .is_empty());
}

#[tokio::test]
#[serial]
async fn test_mair_524_chat_list_since_keeps_the_chats_with_activity() {
    let db = get_smart_db().await;
    let fixtures = get_fixture_db().await;
    let user = plain_user(&fixtures).await;
    let quiet = chat_with(&db, "Calme", vec![user]).await;
    let busy = chat_with(&db, "Animé", vec![user]).await;
    let cursor = db
        .fetch_all::<GetChatsQueryResultView, _>(&GetChatsQueryView::new(user))
        .await
        .unwrap()
        .iter()
        .map(|chat| chat.last_activity_at)
        .max()
        .unwrap();

    let message = post(&db, busy, user, "Du nouveau").await;
    let listed = |since| {
        let db = db.clone();
        async move {
            db.fetch_all::<GetChatsQueryResultView, _>(&GetChatsQueryView::page_since(
                user, 50, 0, None, since,
            ))
            .await
        }
    };
    let ids_since: Vec<u64> = listed(Some(cursor))
        .await
        .unwrap()
        .iter()
        .map(|chat| chat.id as u64)
        .collect();
    assert_eq!(ids_since, vec![busy], "{quiet} had no activity since");
    // Editing and deleting are activity too.
    let after_post = listed(None)
        .await
        .unwrap()
        .into_iter()
        .find(|chat| chat.id as u64 == busy)
        .unwrap()
        .last_activity_at;
    delete(&db, busy, message, user).await.unwrap();
    let after_delete = listed(Some(after_post)).await.unwrap();
    assert_eq!(after_delete.len(), 1);
    assert!(after_delete[0].last_activity_at > after_post);
    // Without `since`, every chat.
    assert_eq!(listed(None).await.unwrap().len(), 2);
}
