use utoipa::ToSchema;

/// A member of a chat.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Core API id of the member: pass it to `GET /api/v1/user/?ids=…` of Core API for the rest of
    /// their profile.
    #[schema(example = 42)]
    id: u64,
    /// First name of the member, as in Core API.
    #[schema(example = "Xavier")]
    first_name: String,
    /// Last name of the member, as in Core API.
    #[schema(example = "Bertrand")]
    last_name: String,
}

impl User {
    pub fn new(id: u64, first_name: &str, last_name: &str) -> Self {
        Self {
            id,
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }
}

/// One page of the members of a chat, by increasing id.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetUsersView {
    /// Members of the chat, by increasing id: at most `limit`. Empty when `offset` is past the
    /// last one.
    users: Vec<User>,
    /// `true` when more members follow: call again with `offset` increased by `limit`.
    #[schema(example = false)]
    has_more: bool,
}

impl GetUsersView {
    pub fn new(users: Vec<User>, has_more: bool) -> Self {
        Self { users, has_more }
    }

    pub fn has_more(&self) -> bool {
        self.has_more
    }

    pub fn users(&self) -> &[User] {
        &self.users
    }
}
