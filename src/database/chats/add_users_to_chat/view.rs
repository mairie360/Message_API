use std::fmt::Display;

use mairie360_api_lib::database::db_interface::DatabaseQueryView;

pub struct AddMembersToChatQueryView {
    chat_id: u64,
    user_id: Vec<i32>,
}

impl AddMembersToChatQueryView {
    pub fn new(chat_id: u64, user_id: Vec<u64>) -> Self {
        Self {
            chat_id,
            user_id: user_id.into_iter().map(|id| id as i32).collect(),
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.chat_id
    }

    pub fn user_id(&self) -> &[i32] {
        &self.user_id
    }
}

impl Display for AddMembersToChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddMembersToChatQueryView: chat_id={} user_id={:?}",
            self.chat_id, self.user_id
        )
    }
}

impl DatabaseQueryView for AddMembersToChatQueryView {
    fn get_request(&self) -> String {
        // $1 est le chat_id, $2 est le tableau des user_ids
        "INSERT INTO conversation_members (conversation_id, user_id)
         SELECT $1, unnest($2::integer[])"
            .to_string()
    }
}
