use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    id: String,
    name: String,
}

impl User {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
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
