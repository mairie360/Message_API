use crate::endpoints::v1::id::get::view::{MessageLinkView, MessageView};
use crate::endpoints::v1::id::messages::tags::MessageTags;
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
        example = "The meeting is moved to 3 pm."
    )]
    content: String,
    /// Core API ids of the members the message mentions (MAIR-1155): non-excluded members of the
    /// chat, at most 50, no duplicate. Absent or `null`: none.
    #[schema(example = json!([51]), max_items = 50)]
    #[serde(default)]
    mentions: Option<Vec<u64>>,
    /// Whether the message mentions every member of the chat (`@everyone`, MAIR-1155). Absent or `null`: none.
    #[schema(example = false)]
    #[serde(default)]
    mentions_everyone: Option<bool>,
    /// Ids of the groups the message mentions (MAIR-1155): groups at least one member of the chat
    /// belongs to (`GET /api/v1/{chat_id}/mention-groups/`), at most 50, no duplicate. Their
    /// members present in the chat are notified (`MENTIONED`). Absent or `null`: none.
    #[schema(example = json!([]), max_items = 50)]
    #[serde(default)]
    group_mentions: Option<Vec<u64>>,
    /// Business objects the message links to (MAIR-1155): projects, tasks or events the author
    /// may read, at most 50, no duplicate. Absent or `null`: none.
    #[schema(max_items = 50)]
    #[serde(default)]
    links: Option<Vec<MessageLinkView>>,
}

impl PostMessageView {
    pub fn new(citation: Option<u64>, content: String) -> Self {
        Self {
            citation,
            content,
            mentions: None,
            mentions_everyone: None,
            group_mentions: None,
            links: None,
        }
    }

    /// Mentions and links of the message.
    pub fn tags(&self) -> MessageTags {
        MessageTags {
            mentions: self.mentions.clone(),
            mentions_everyone: self.mentions_everyone,
            group_mentions: self.group_mentions.clone(),
            links: self.links.clone(),
        }
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
    /// The message as `GET /api/v1/{chat_id}/` reads it, with the `quoted` message it answers, so the
    /// author can show it at once.
    message: MessageView,
}

impl PostMessageResultView {
    pub fn new(message: MessageView) -> Self {
        Self {
            id: message.id(),
            message,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn message(&self) -> &MessageView {
        &self.message
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
        check_description("content", &self.content, MAX_MESSAGE_LENGTH)?;
        self.tags().validate()
    }
}
