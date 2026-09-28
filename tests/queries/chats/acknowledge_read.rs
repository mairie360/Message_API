use crate::common::{get_smart_db, plain_user};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};
use mairie360_api_lib::database::error::DbError;
use mairie360_api_lib::error::ApiLibError;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    acknowledge_read::view::AcknowledgeReadQueryView,
    add_message_to_chat::view::PostMessageInChatQueryView,
    add_users_to_chat::view::AddMembersToChatQueryView,
    create_chat::view::CreateChatQueryView,
    get_chats::view::{GetChatsQueryResultView, GetChatsQueryView},
    remove_user_from_chat::view::RemoveMemberFromChatQueryView,
};
use serial_test::serial;
use std::fmt::Display;

/// Highest message id of a chat.
#[derive(serde::Serialize, serde::Deserialize)]
struct LastMessageId {
    params: Vec<QueryParam>,
}

impl LastMessageId {
    fn new(chat_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I32(chat_id as i32)],
        }
    }
}

impl Display for LastMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LastMessageId: {:?}", self.params)
    }
}

impl ApiRequestDto for LastMessageId {
    fn query_sql(&self) -> &'static str {
        "SELECT COALESCE(MAX(id), 0) FROM messages WHERE conversation_id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// What the unread count of `user_id` must be according to their read cursor, recomputed from the
/// messages: the ones written by someone else after the cursor.
#[derive(serde::Serialize, serde::Deserialize)]
struct UnreadAfterCursor {
    params: Vec<QueryParam>,
}

impl UnreadAfterCursor {
    fn new(chat_id: u64, user_id: u64) -> Self {
        Self {
            params: vec![
                QueryParam::I32(chat_id as i32),
                QueryParam::I32(user_id as i32),
            ],
        }
    }
}

impl Display for UnreadAfterCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnreadAfterCursor: {:?}", self.params)
    }
}

impl ApiRequestDto for UnreadAfterCursor {
    fn query_sql(&self) -> &'static str {
        "SELECT count(*) FROM messages m \
         WHERE m.conversation_id = $1 AND m.owner_id IS DISTINCT FROM $2 \
           AND m.id > COALESCE((SELECT last_read_message_id FROM conversation_read_cursors \
                                WHERE conversation_id = $1 AND user_id = $2), 0)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

async fn chat_with(db: &SmartDatabase, members: Vec<u64>) -> u64 {
    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Acknowledge", None))
        .await
        .unwrap() as u64;
    db.execute(AddMembersToChatQueryView::new(chat_id, members))
        .await
        .unwrap();
    chat_id
}

async fn post(db: &SmartDatabase, chat_id: u64, sender: u64) -> u64 {
    db.fetch_scalar::<i64, _>(&PostMessageInChatQueryView::new(chat_id, sender, "hello"))
        .await
        .unwrap() as u64
}

async fn acknowledge(
    db: &SmartDatabase,
    chat_id: u64,
    user_id: u64,
    message_id: u64,
) -> Result<i32, ApiLibError> {
    db.fetch_scalar::<i32, _>(&AcknowledgeReadQueryView::new(chat_id, user_id, message_id))
        .await
}

/// The `unread_count` `GET /api/v1/` reports for the chat.
async fn listed_unread(db: &SmartDatabase, chat_id: u64, user_id: u64) -> i32 {
    db.fetch_all::<GetChatsQueryResultView, _>(&GetChatsQueryView::new(user_id))
        .await
        .unwrap()
        .into_iter()
        .find(|chat| chat.id as u64 == chat_id)
        .map_or(0, |chat| chat.unread_count)
}

#[tokio::test]
#[serial]
async fn test_acknowledge_read_only_clears_up_to_the_cursor() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let sender = plain_user(&db).await;
    let reader = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![sender, reader]).await;
    let first = post(&db, chat_id, sender).await;
    let second = post(&db, chat_id, sender).await;
    let third = post(&db, chat_id, sender).await;
    assert_eq!(listed_unread(&db, chat_id, reader).await, 3);

    assert_eq!(acknowledge(&db, chat_id, reader, second).await.unwrap(), 1);
    assert_eq!(listed_unread(&db, chat_id, reader).await, 1);

    // A stale cursor neither resurrects the read messages nor moves the cursor back.
    assert_eq!(acknowledge(&db, chat_id, reader, first).await.unwrap(), 1);
    assert_eq!(listed_unread(&db, chat_id, reader).await, 1);

    assert_eq!(acknowledge(&db, chat_id, reader, third).await.unwrap(), 0);
    assert_eq!(listed_unread(&db, chat_id, reader).await, 0);
    // Idempotent.
    assert_eq!(acknowledge(&db, chat_id, reader, third).await.unwrap(), 0);
}

#[tokio::test]
#[serial]
async fn test_stale_acknowledgement_cannot_clear_a_new_message() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let sender = plain_user(&db).await;
    let reader = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![sender, reader]).await;
    let displayed = post(&db, chat_id, sender).await;

    // The agent displays `displayed`, then a new message arrives while the thread is open.
    let arrived_meanwhile = post(&db, chat_id, sender).await;
    assert_eq!(
        acknowledge(&db, chat_id, reader, displayed).await.unwrap(),
        1
    );
    assert_eq!(listed_unread(&db, chat_id, reader).await, 1);

    // Repeating the same (now stale) acknowledgement keeps it unread.
    assert_eq!(
        acknowledge(&db, chat_id, reader, displayed).await.unwrap(),
        1
    );
    assert_eq!(listed_unread(&db, chat_id, reader).await, 1);

    assert_eq!(
        acknowledge(&db, chat_id, reader, arrived_meanwhile)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
#[serial]
async fn test_acknowledge_read_refuses_a_message_of_another_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let sender = plain_user(&db).await;
    let reader = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![sender, reader]).await;
    let other_chat_id = chat_with(&db, vec![sender, reader]).await;
    post(&db, chat_id, sender).await;
    let foreign = post(&db, other_chat_id, sender).await;

    let result = acknowledge(&db, chat_id, reader, foreign).await;
    assert!(
        matches!(result, Err(ApiLibError::Database(DbError::NotFound))),
        "a foreign message must be refused, got: {result:?}"
    );
    // Nothing was acknowledged, in either chat.
    assert_eq!(listed_unread(&db, chat_id, reader).await, 1);
    assert_eq!(listed_unread(&db, other_chat_id, reader).await, 1);

    let unknown = acknowledge(&db, chat_id, reader, i64::MAX as u64).await;
    assert!(matches!(
        unknown,
        Err(ApiLibError::Database(DbError::NotFound))
    ));
}

#[tokio::test]
#[serial]
async fn test_acknowledge_read_never_counts_own_or_removed_members() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;
    let sender = plain_user(&db).await;
    let removed = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![sender, removed]).await;
    db.execute(RemoveMemberFromChatQueryView::new(chat_id, removed))
        .await
        .unwrap();
    let message = post(&db, chat_id, sender).await;

    assert_eq!(acknowledge(&db, chat_id, sender, message).await.unwrap(), 0);
    assert_eq!(
        acknowledge(&db, chat_id, removed, message).await.unwrap(),
        0
    );
    assert_eq!(listed_unread(&db, chat_id, removed).await, 0);
}

// Reading while messages keep arriving: whatever the interleaving, no acknowledgement may erase a
// message posted after the cursor (the counter must stay equal to what the cursor implies), and
// once the traffic stops one last acknowledgement clears everything.
#[tokio::test]
#[serial]
async fn test_acknowledge_read_races_with_new_messages() {
    let (_container, host) = get_shared_db().await;
    let host = host.to_string();
    let db = get_smart_db(&host).await;
    let sender = plain_user(&db).await;
    let reader = plain_user(&db).await;
    let chat_id = chat_with(&db, vec![sender, reader]).await;
    post(&db, chat_id, sender).await;

    let senders: Vec<_> = (0..4)
        .map(|_| {
            let host = host.clone();
            tokio::spawn(async move {
                let db = get_smart_db(&host).await;
                for _ in 0..25 {
                    post(&db, chat_id, sender).await;
                }
            })
        })
        .collect();
    let reader_task = {
        let host = host.clone();
        tokio::spawn(async move {
            let db = get_smart_db(&host).await;
            for _ in 0..60 {
                // The agent acknowledges the newest message it can see.
                let last = db
                    .fetch_scalar::<i64, _>(&LastMessageId::new(chat_id))
                    .await
                    .unwrap() as u64;
                acknowledge(&db, chat_id, reader, last).await.unwrap();
            }
        })
    };
    for task in senders {
        task.await.unwrap();
    }
    reader_task.await.unwrap();

    let expected = db
        .fetch_scalar::<i64, _>(&UnreadAfterCursor::new(chat_id, reader))
        .await
        .unwrap() as i32;
    assert_eq!(
        listed_unread(&db, chat_id, reader).await,
        expected,
        "the counter must match the messages after the read cursor"
    );

    let last = db
        .fetch_scalar::<i64, _>(&LastMessageId::new(chat_id))
        .await
        .unwrap() as u64;
    assert_eq!(acknowledge(&db, chat_id, reader, last).await.unwrap(), 0);
    assert_eq!(listed_unread(&db, chat_id, reader).await, 0);
}
