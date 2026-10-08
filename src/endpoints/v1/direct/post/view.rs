use crate::endpoints::validation::{check_user_ids, Validate, ValidationError};
use utoipa::ToSchema;

/// Agent to open the direct chat with.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct OpenDirectChatView {
    /// Core API id of the other participant: between 1 and 2147483647, not the caller.
    #[schema(minimum = 1, maximum = 2147483647, example = 42)]
    contact_id: u64,
}

impl OpenDirectChatView {
    pub fn new(contact_id: u64) -> Self {
        Self { contact_id }
    }

    pub fn contact_id(&self) -> u64 {
        self.contact_id
    }
}

impl Validate for OpenDirectChatView {
    fn validate(&self) -> Result<(), ValidationError> {
        check_user_ids("contact_id", &[self.contact_id], 1, false)
    }
}

/// The direct chat of the caller and the contact.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct OpenDirectChatResultView {
    /// Id of the direct chat, to use as `chat_id` in `/api/v1/{chat_id}/` (post the message with
    /// `POST /api/v1/{chat_id}/messages/`).
    #[schema(example = 9)]
    id: u64,
    /// `true` when this call created the chat, `false` when the pair already had one (its
    /// history is kept).
    #[schema(example = false)]
    created: bool,
}

impl OpenDirectChatResultView {
    pub fn new(id: u64, created: bool) -> Self {
        Self { id, created }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn created(&self) -> bool {
        self.created
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_must_be_a_user_id() {
        assert!(OpenDirectChatView::new(42).validate().is_ok());
        for id in [0, i32::MAX as u64 + 1] {
            let error = OpenDirectChatView::new(id).validate().unwrap_err();
            assert!(error.to_string().contains("contact_id"), "{error}");
        }
    }
}
