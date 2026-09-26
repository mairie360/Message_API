use crate::endpoints::validation::{
    check_description, Validate, ValidationError, MAX_MESSAGE_LENGTH,
};
use utoipa::ToSchema;

/// Message à publier dans la conversation du chemin.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct PostMessageView {
    /// Identifiant du message auquel celui-ci répond. Facultatif.
    #[schema(example = 100)]
    sitation: Option<u64>, // message sitation
    /// Contenu du message.
    #[schema(
        min_length = 1,
        max_length = 5000,
        example = "La réunion est décalée à 15h."
    )]
    content: String,
}

impl PostMessageView {
    pub fn new(sitation: Option<u64>, content: String) -> Self {
        Self { sitation, content }
    }

    pub fn sitation(&self) -> Option<u64> {
        self.sitation
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

/// Message publié.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PostMessageResultView {
    /// Identifiant attribué au message publié.
    #[schema(example = 101)]
    id: u64,
}

impl PostMessageResultView {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl Validate for PostMessageView {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.content.trim().is_empty() {
            return Err(ValidationError::new("content", "must not be empty"));
        }
        check_description("content", &self.content, MAX_MESSAGE_LENGTH)
    }
}
