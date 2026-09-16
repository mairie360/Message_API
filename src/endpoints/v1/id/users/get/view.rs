use utoipa::ToSchema;

/// Participant d'une conversation.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct User {
    /// Identifiant Core API du participant, à repasser à `GET /api/v1/user/?ids=…` de Core API
    /// pour obtenir sa fiche.
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

/// Participants d'une conversation.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetUsersView {
    /// Participants de la conversation. Vide si la conversation n'existe pas.
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
