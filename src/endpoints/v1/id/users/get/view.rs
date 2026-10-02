use utoipa::ToSchema;

/// A member of a chat.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Core API id of the member: pass it to `GET /api/v1/user/?ids=…` of Core API to get their
    /// profile.
    #[schema(example = 42)]
    id: u64,
}

impl User {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
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
