use utoipa::ToSchema;

use crate::endpoints::v1::id::get::view::ReactionView;
use crate::endpoints::validation::{check_emoji, Validate, ValidationError};

/// The reaction to add.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct AddReactionView {
    /// The emoji: 1 to 32 characters, no whitespace nor control character.
    #[schema(example = "👍", min_length = 1, max_length = 32)]
    emoji: String,
}

impl AddReactionView {
    pub fn new(emoji: &str) -> Self {
        Self {
            emoji: emoji.to_string(),
        }
    }

    pub fn emoji(&self) -> &str {
        &self.emoji
    }
}

impl Validate for AddReactionView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_emoji("emoji", &self.emoji)
    }
}

/// The reactions of the message after the change.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ReactionsView {
    /// One entry per emoji, in the order of its first reaction (same as the message's
    /// `reactions` in `GET /api/v1/{chat_id}/`).
    reactions: Vec<ReactionView>,
}

impl ReactionsView {
    pub fn new(reactions: Vec<ReactionView>) -> Self {
        Self { reactions }
    }

    pub fn reactions(&self) -> &[ReactionView] {
        &self.reactions
    }
}
