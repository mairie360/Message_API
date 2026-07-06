use crate::common::get_pool; // Utilisation de ta fonction utilitaire existante
use mairie360_api_lib::test_setup::queries_setup::get_shared_db;
use message_api::database::chats::{
    add_message_to_chat::{query::post_message_in_chat_query, view::PostMessageInChatQueryView},
    create_chat::{query::create_chat_query, view::CreateChatQueryView},
    patch_message_in_chat::{query::patch_message_query, view::PatchMessageQueryView},
};

#[sqlx::test]
async fn test_patch_message_success() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = CreateChatQueryView::new("Test Chat", None);

    let result = create_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());

    let view = PostMessageInChatQueryView::new(result.unwrap() as u64, 1, "Test Message");
    let result = post_message_in_chat_query(view, pool.clone()).await;

    assert!(result.is_ok());

    let view = PatchMessageQueryView::new(result.unwrap() as u64, "Updated Content");
    let result = patch_message_query(view, pool).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[sqlx::test]
async fn test_patch_message_unknown_message() {
    let (_container, host) = get_shared_db().await;
    let pool = get_pool(host.to_string()).await;

    let view = PatchMessageQueryView::new(999, "Updated Content");
    let result = patch_message_query(view, pool).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}
