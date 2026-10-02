use crate::endpoints::validation::{
    check_user_ids, Validate, ValidationError, MAX_MEMBERS_PER_REQUEST,
};
use utoipa::ToSchema;

/// Users to attach to the chat of the path.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddUsersToChat {
    /// Core API ids of the users to attach, in one request: 1 to 50 distinct ids, each between 1
    /// and 2147483647. None of them may already be a member.
    #[schema(example = json!([42, 51]), min_items = 1, max_items = 50)]
    pub users_id: Vec<u64>,
}

impl AddUsersToChat {
    pub fn new(users_id: Vec<u64>) -> Self {
        Self { users_id }
    }

    pub fn users_id(&self) -> &[u64] {
        &self.users_id
    }
}

impl Validate for AddUsersToChat {
    fn validate(&self) -> Result<(), ValidationError> {
        check_user_ids("users_id", &self.users_id, MAX_MEMBERS_PER_REQUEST, false)
    }
}

/// Result of the attachment: the users actually added.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct AddUsersToChatResultView {
    /// The chat the users were added to.
    #[schema(example = 5)]
    chat_id: u64,
    /// Ids actually attached: always every id of `users_id`, since the request is all or nothing.
    #[schema(example = json!([42, 51]))]
    added: Vec<u64>,
}

impl AddUsersToChatResultView {
    pub fn new(chat_id: u64, added: Vec<u64>) -> Self {
        Self { chat_id, added }
    }
}
