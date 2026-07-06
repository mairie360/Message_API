use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
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

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetUsersView {
    users: Vec<User>,
}

impl GetUsersView {
    pub fn new(users: Vec<User>) -> Self {
        Self { users }
    }

    pub fn users(&self) -> &[User] {
        &self.users
    }
}
