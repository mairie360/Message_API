use crate::endpoints::validation::{
    check_label, check_user_ids, Validate, ValidationError, MAX_MEMBERS_PER_REQUEST,
    MAX_TITLE_LENGTH,
};
use utoipa::ToSchema;

/// Chat to create, with its first members.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateChatView {
    /// Core API ids of the other members: at most 50 distinct ids, each between 1 and 2147483647.
    /// May be empty. The caller is added automatically (listing them is harmless).
    #[schema(example = json!([42, 51]), max_items = 50)]
    members: Vec<u64>,
    /// Title of the chat: empty for a direct chat, otherwise 1 to 150 characters.
    #[schema(max_length = 150, example = "Service urbanisme")]
    name: String,
}

impl CreateChatView {
    pub fn new(members: Vec<u64>, name: String) -> Self {
        Self { members, name }
    }

    pub fn members(&self) -> &[u64] {
        &self.members
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Chat created.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateChatResultView {
    /// Id given to the chat.
    #[schema(example = 5)]
    id: u64,
}

impl CreateChatResultView {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl Validate for CreateChatView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_user_ids("members", &self.members, MAX_MEMBERS_PER_REQUEST, true)?;
        // An empty title is allowed (direct conversation); a non-empty one is a displayed label.
        if self.name.is_empty() {
            return Ok(());
        }
        check_label("name", &self.name, MAX_TITLE_LENGTH)
    }
}
