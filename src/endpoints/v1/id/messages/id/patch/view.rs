use crate::endpoints::v1::id::get::view::MessageLinkView;
use crate::endpoints::v1::id::messages::tags::MessageTags;
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
    /// Core API ids of the members the message mentions (MAIR-1155): non-excluded members of the
    /// chat, at most 50, no duplicate. Absent: kept as is; present (even empty): replaces it.
    #[schema(example = json!([51]), max_items = 50)]
    #[serde(default)]
    mentions: Option<Vec<u64>>,
    /// Whether the message mentions every member of the chat (`@everyone`, MAIR-1155). Absent: kept as is; present (even empty): replaces it.
    #[schema(example = false)]
    #[serde(default)]
    mentions_everyone: Option<bool>,
    /// Ids of the groups the message mentions (MAIR-1155): groups at least one member of the chat
    /// belongs to (`GET /api/v1/{chat_id}/mention-groups/`), at most 50, no duplicate. Their
    /// members present in the chat are notified (`MENTIONED`). Absent: kept as is; present (even empty): replaces it.
    #[schema(example = json!([]), max_items = 50)]
    #[serde(default)]
    group_mentions: Option<Vec<u64>>,
    /// Business objects the message links to (MAIR-1155): projects, tasks or events the author
    /// may read, at most 50, no duplicate. Absent: kept as is; present (even empty): replaces it.
    #[schema(max_items = 50)]
    #[serde(default)]
    links: Option<Vec<MessageLinkView>>,
}

impl PatchMessageView {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            mentions: None,
            mentions_everyone: None,
            group_mentions: None,
            links: None,
        }
    }

    /// The tags to replace (`None` = keep).
    pub fn tags(&self) -> MessageTags {
        MessageTags {
            mentions: self.mentions.clone(),
            mentions_everyone: self.mentions_everyone,
            group_mentions: self.group_mentions.clone(),
            links: self.links.clone(),
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
        check_description("content", &self.content, MAX_MESSAGE_LENGTH)?;
        self.tags().validate()
    }
}
