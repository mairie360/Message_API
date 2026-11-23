use utoipa::IntoParams;

use crate::endpoints::validation::{check_emoji, Validate, ValidationError};

/// The reaction to remove.
#[derive(Debug, serde::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RemoveReactionQuery {
    /// The emoji of the caller's reaction, URL-encoded (`%F0%9F%91%8D` for 👍).
    #[param(example = "👍", min_length = 1, max_length = 32)]
    emoji: String,
}

impl RemoveReactionQuery {
    pub fn new(emoji: &str) -> Self {
        Self {
            emoji: emoji.to_string(),
        }
    }

    pub fn emoji(&self) -> &str {
        &self.emoji
    }
}

impl Validate for RemoveReactionQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        check_emoji("emoji", &self.emoji)
    }
}
