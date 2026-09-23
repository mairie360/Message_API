use crate::endpoints::validation::{
    check_description, Validate, ValidationError, MAX_MESSAGE_LENGTH,
};

/// Nouveau contenu d'un message, qui remplace entièrement l'ancien.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct PatchMessageView {
    /// Nouveau contenu, qui remplace intégralement l'ancien. Obligatoire.
    #[schema(
        min_length = 1,
        max_length = 5000,
        example = "La réunion est finalement décalée à 16h."
    )]
    content: String,
}

impl PatchMessageView {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Validate for PatchMessageView {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.content.trim().is_empty() {
            return Err(ValidationError::new("content", "must not be empty"));
        }
        check_description("content", &self.content, MAX_MESSAGE_LENGTH)
    }
}
