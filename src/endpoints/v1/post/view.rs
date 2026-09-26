use crate::endpoints::validation::{check_label, Validate, ValidationError, MAX_TITLE_LENGTH};
use utoipa::ToSchema;

/// Conversation à créer, avec ses participants initiaux.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateChatView {
    /// Identifiants Core API des autres participants. L'appelant est ajouté automatiquement :
    /// ne pas inclure son propre identifiant, ni de doublon.
    #[schema(example = json!([42, 51]))]
    members: Vec<u64>,
    /// Titre de la conversation.
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

/// Conversation créée.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct CreateChatResultView {
    /// Identifiant attribué à la conversation créée.
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
        // An empty title is allowed (direct conversation); a non-empty one is a displayed label.
        if self.name.is_empty() {
            return Ok(());
        }
        check_label("name", &self.name, MAX_TITLE_LENGTH)
    }
}
