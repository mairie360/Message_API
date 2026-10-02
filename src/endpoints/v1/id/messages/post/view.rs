use crate::endpoints::validation::{
    check_description, Validate, ValidationError, MAX_MESSAGE_ID, MAX_MESSAGE_LENGTH,
};
use utoipa::ToSchema;

/// Message to post in the chat of the path.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct PostMessageView {
    /// Id of the message this one answers, in the same chat. Optional (`null` or absent).
    #[schema(example = 118, minimum = 1)]
    citation: Option<u64>,
    /// Content of the message.
    #[schema(
        min_length = 1,
        max_length = 5000,
        example = "La réunion est décalée à 15h."
    )]
    content: String,
}

impl PostMessageView {
    pub fn new(citation: Option<u64>, content: String) -> Self {
        Self { citation, content }
    }

    pub fn citation(&self) -> Option<u64> {
        self.citation
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

/// Message posted.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct PostMessageResultView {
    /// Id given to the message.
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
        if matches!(self.citation, Some(id) if id == 0 || id > MAX_MESSAGE_ID) {
            return Err(ValidationError::new(
                "citation",
                &format!("must be a message id between 1 and {MAX_MESSAGE_ID}"),
            ));
        }
        check_description("content", &self.content, MAX_MESSAGE_LENGTH)
    }
}
