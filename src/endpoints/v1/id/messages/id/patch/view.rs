use crate::endpoints::validation::{
    check_description, Validate, ValidationError, MAX_MESSAGE_LENGTH,
};

/// New content of a message, which replaces the old one entirely.
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct PatchMessageView {
    /// New content, which replaces the old one entirely. Required.
    #[schema(
        min_length = 1,
        max_length = 5000,
        example = "The meeting is finally moved to 4 pm."
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
