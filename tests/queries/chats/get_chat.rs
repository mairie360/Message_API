use crate::common::get_smart_db;
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_message_to_chat::view::PostMessageInChatQueryView,
    create_chat::view::CreateChatQueryView,
    get_chat::view::{GetChatQueryView, Message},
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_get_chat_success() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = CreateChatQueryView::new("Test Chat", None);
    let chat_id = db.fetch_scalar::<i32, _>(&view).await.unwrap() as u64;

    for _ in 0..3 {
        let view = PostMessageInChatQueryView::new(chat_id, 1, "Test Message");
        let _ = db.fetch_scalar::<i64, _>(&view).await;
    }

    let view = GetChatQueryView::new(chat_id, None, 50);
    let result = db.fetch_all::<Message, _>(&view).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 3);
}

#[tokio::test]
#[serial]
async fn test_get_chat_unknown_chat() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let view = GetChatQueryView::new(999, None, 50);
    let result = db.fetch_all::<Message, _>(&view).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
#[serial]
async fn test_get_chat_pages_newest_first_with_a_cursor() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Pages", None))
        .await
        .unwrap() as u64;
    let mut ids = Vec::new();
    for n in 0..5 {
        let view = PostMessageInChatQueryView::new(chat_id, 1, &format!("message {n}"));
        ids.push(db.fetch_scalar::<i64, _>(&view).await.unwrap());
    }

    let first: Vec<i64> = db
        .fetch_all::<Message, _>(&GetChatQueryView::new(chat_id, None, 2))
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(first, vec![ids[4], ids[3]]);

    let second: Vec<i64> = db
        .fetch_all::<Message, _>(&GetChatQueryView::new(chat_id, Some(ids[3] as u64), 2))
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(second, vec![ids[2], ids[1]]);

    let last: Vec<i64> = db
        .fetch_all::<Message, _>(&GetChatQueryView::new(chat_id, Some(ids[1] as u64), 2))
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(last, vec![ids[0]]);
}

#[tokio::test]
#[serial]
async fn test_get_chat_reads_the_citation_back() {
    let (_container, host) = get_shared_db().await;
    let db = get_smart_db(host).await;

    let chat_id = db
        .fetch_scalar::<i32, _>(&CreateChatQueryView::new("Citations", None))
        .await
        .unwrap() as u64;
    let quoted = db
        .fetch_scalar::<i64, _>(&PostMessageInChatQueryView::new(chat_id, 1, "Question"))
        .await
        .unwrap();
    let reply = db
        .fetch_scalar::<i64, _>(&PostMessageInChatQueryView::replying_to(
            chat_id,
            1,
            "Answer",
            Some(quoted as u64),
        ))
        .await
        .unwrap();

    let messages = db
        .fetch_all::<Message, _>(&GetChatQueryView::new(chat_id, None, 50))
        .await
        .unwrap();
    let reply = messages.iter().find(|m| m.id == reply).unwrap();
    assert_eq!(reply.reply_to_id, Some(quoted));
    assert_eq!(reply.owner_id, Some(1));
}
