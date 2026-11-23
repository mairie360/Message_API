//! System notes (MAIR-1155): the messages a member change writes in a group chat
//! (`messages.kind = 'system'`, `system_event` = what happened, empty content). Database keeps
//! them out of the unread counters.

use std::fmt::Display;

use crate::database::ids::{id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// What a system note records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAction {
    /// The actor added `user_ids`.
    MembersAdded,
    /// The actor removed `user_ids` (one member).
    MemberRemoved,
    /// The actor left the chat (`user_ids` = the actor).
    MemberLeft,
}

impl SystemAction {
    /// Value of `system_event.action`.
    pub fn as_str(self) -> &'static str {
        match self {
            SystemAction::MembersAdded => "members_added",
            SystemAction::MemberRemoved => "member_removed",
            SystemAction::MemberLeft => "member_left",
        }
    }
}

/// Writes the system note of `action`, performed by `actor_id` on `user_ids`, in chat `chat_id`,
/// and returns its id; no row when nobody sees the chat any more (its last member left: the chat
/// is deleted next).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostSystemNoteQueryView {
    params: Vec<QueryParam>,
}

impl PostSystemNoteQueryView {
    pub fn new(chat_id: u64, actor_id: u64, action: SystemAction, user_ids: &[u64]) -> Self {
        let event = serde_json::json!({
            "action": action.as_str(),
            "actor_id": id_to_sql(actor_id),
            "user_ids": user_ids.iter().map(|id| id_to_sql(*id)).collect::<Vec<i32>>(),
        });
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::I32(id_to_sql(actor_id)),
                QueryParam::Text(event.to_string()),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn actor_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl Display for PostSystemNoteQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PostSystemNoteQueryView: chat_id={} actor_id={}",
            self.chat_id(),
            self.actor_id()
        )
    }
}

impl ApiRequestDto for PostSystemNoteQueryView {
    fn query_sql(&self) -> &'static str {
        "INSERT INTO messages (conversation_id, owner_id, content, kind, system_event) \
         SELECT $1, $2, '', 'system', $3::jsonb \
         WHERE EXISTS (SELECT 1 FROM conversation_members \
                       WHERE conversation_id = $1 AND is_excluded = FALSE) \
         RETURNING to_jsonb(id)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
