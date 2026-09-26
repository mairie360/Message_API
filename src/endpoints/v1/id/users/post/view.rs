use utoipa::ToSchema;

/// Utilisateurs à rattacher à la conversation du chemin.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddUsersToChat {
    /// Identifiants Core API des utilisateurs à rattacher, en une seule requête.
    #[schema(example = json!([42, 51]))]
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

/// Résultat du rattachement : utilisateurs effectivement ajoutés.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct AddUsersToChatResultView {
    /// Conversation concernée.
    #[schema(example = 5)]
    chat_id: u64,
    /// Identifiants effectivement rattachés. Les comparer à `users_id` pour repérer ceux qui
    /// étaient déjà participants ou qui n'existent pas.
    #[schema(example = json!([42, 51]))]
    added: Vec<u64>,
}

impl AddUsersToChatResultView {
    pub fn new(chat_id: u64, added: Vec<u64>) -> Self {
        Self { chat_id, added }
    }
}
