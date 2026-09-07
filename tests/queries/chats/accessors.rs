//! Tests unitaires (sans base de données) des `QueryView` de la couche `chats` :
//! on vérifie que les constructeurs, les getters, l'implémentation `Display` et
//! le contrat `ApiRequestDto` (SQL + params) restent cohérents.

use mairie360_api_lib::database::db_interface::ApiRequestDto;
use message_api::database::chats::{
    add_message_to_chat::view::PostMessageInChatQueryView,
    add_users_to_chat::view::AddMembersToChatQueryView, create_chat::view::CreateChatQueryView,
    delete_chat::view::DeleteChatQueryView, delete_message_from_chat::view::DeleteMessageQueryView,
    get_chat::view::GetChatQueryView, get_chat_users::view::GetChatMembersQueryView,
    get_chats::view::GetChatsQueryView, patch_message_in_chat::view::PatchMessageQueryView,
    remove_user_from_chat::view::RemoveMemberFromChatQueryView,
    reset_unread_count::view::ResetUnreadCountQueryView,
};

#[test]
fn test_post_message_in_chat_view_accessors() {
    let view = PostMessageInChatQueryView::new(12, 34, "hello");

    assert_eq!(view.chat_id(), 12);
    assert_eq!(view.sender(), 34);
    assert_eq!(view.message(), "hello");
    assert!(format!("{view}").contains("chat_id=12"));
    assert!(view.query_sql().contains("INSERT INTO messages"));
    assert_eq!(view.query_params().len(), 3);
}

#[test]
fn test_add_members_to_chat_view_accessors() {
    let view = AddMembersToChatQueryView::new(7, vec![1, 2, 3]);

    assert_eq!(view.chat_id(), 7);
    assert_eq!(view.user_id(), vec![1, 2, 3]);
    assert!(!view.is_empty());
    assert!(format!("{view}").contains("[1, 2, 3]"));
    assert!(view.query_sql().contains("conversation_members"));
    assert_eq!(view.query_params().len(), 2);
}

#[test]
fn test_add_members_to_chat_view_empty_list() {
    let view = AddMembersToChatQueryView::new(7, vec![]);

    assert!(view.user_id().is_empty());
    assert!(view.is_empty());
}

#[test]
fn test_create_chat_view_accessors() {
    let view = CreateChatQueryView::new("Title", Some(9));

    assert_eq!(view.title(), "Title");
    assert_eq!(view.group_id(), Some(9));
    assert!(format!("{view}").contains("group_id=9"));
    assert!(view.query_sql().contains("INSERT INTO conversations"));
    assert_eq!(view.query_params().len(), 2);

    let without_group = CreateChatQueryView::new("Title", None);
    assert_eq!(without_group.group_id(), None);
    assert!(format!("{without_group}").contains("group_id=0"));
}

#[test]
fn test_delete_chat_view_accessors() {
    let view = DeleteChatQueryView::new(42);

    assert_eq!(view.chat_id(), 42);
    assert!(format!("{view}").contains("chat_id=42"));
    assert!(view.query_sql().contains("DELETE FROM conversations"));
    assert_eq!(view.query_params().len(), 1);
}

#[test]
fn test_delete_message_view_accessors() {
    let view = DeleteMessageQueryView::new(42);

    assert_eq!(view.message_id(), 42);
    assert!(format!("{view}").contains("message_id=42"));
    assert!(view.query_sql().contains("DELETE FROM messages"));
    assert_eq!(view.query_params().len(), 1);
}

#[test]
fn test_get_chat_view_accessors() {
    let view = GetChatQueryView::new(42);

    assert_eq!(view.chat_id(), 42);
    assert!(format!("{view}").contains("chat_id=42"));
    assert!(view.query_sql().contains("FROM messages"));
    assert_eq!(view.query_params().len(), 1);
}

#[test]
fn test_get_chat_members_view_accessors() {
    let view = GetChatMembersQueryView::new(42);

    assert_eq!(view.chat_id(), 42);
    assert!(format!("{view}").contains("chat_id=42"));
    assert!(view.query_sql().contains("conversation_members"));
    assert_eq!(view.query_params().len(), 1);
}

#[test]
fn test_get_chats_view_accessors() {
    let view = GetChatsQueryView::new(42);

    assert_eq!(view.user_id(), 42);
    assert!(format!("{view}").contains("user_id=42"));
    assert!(view.query_sql().contains("FROM conversations"));
    assert_eq!(view.query_params().len(), 1);
}

#[test]
fn test_patch_message_view_accessors() {
    let view = PatchMessageQueryView::new(42, "new content");

    assert_eq!(view.message_id(), 42);
    assert_eq!(view.content(), "new content");
    assert!(format!("{view}").contains("message_id=42"));
    assert!(view.query_sql().contains("UPDATE messages"));
    assert_eq!(view.query_params().len(), 2);
}

#[test]
fn test_remove_member_from_chat_view_accessors() {
    let view = RemoveMemberFromChatQueryView::new(7, 42);

    assert_eq!(view.chat_id(), 7);
    assert_eq!(view.user_id(), 42);
    assert!(format!("{view}").contains("chat_id=7 user_id=42"));
    assert!(view
        .query_sql()
        .contains("DELETE FROM conversation_members"));
    assert_eq!(view.query_params().len(), 2);
}

#[test]
fn test_reset_unread_count_view_accessors() {
    let view = ResetUnreadCountQueryView::new(7, 42);

    assert_eq!(view.chat_id(), 7);
    assert_eq!(view.user_id(), 42);
    assert!(format!("{view}").contains("chat_id=7 user_id=42"));
    assert!(view.query_sql().contains("UPDATE unread_counters"));
    assert_eq!(view.query_params().len(), 2);
}
