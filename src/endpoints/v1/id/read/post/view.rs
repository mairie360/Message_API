use crate::endpoints::validation::{Validate, ValidationError};
use utoipa::ToSchema;

/// Read acknowledgement of the chat of the path.
#[derive(Debug, serde::Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgeReadView {
    /// Id of the last message the agent has actually displayed, taken from the messages returned
    /// by `GET /api/v1/{chat_id}/`. Every message up to and including it is acknowledged; later
    /// ones stay unread. It must belong to the chat of the path.
    #[schema(minimum = 1, format = Int64, example = 118)]
    read_until_message_id: u64,
}

impl AcknowledgeReadView {
    pub fn new(read_until_message_id: u64) -> Self {
        Self {
            read_until_message_id,
        }
    }

    pub fn read_until_message_id(&self) -> u64 {
        self.read_until_message_id
    }
}

impl Validate for AcknowledgeReadView {
    fn validate(&self) -> Result<(), ValidationError> {
        // `messages.id` is a BIGINT: a larger value would be rejected by Postgres as a 500.
        if self.read_until_message_id == 0 || self.read_until_message_id > i64::MAX as u64 {
            return Err(ValidationError::new(
                "readUntilMessageId",
                "must be a message id between 1 and 9223372036854775807",
            ));
        }
        Ok(())
    }
}

/// State of the chat after the acknowledgement.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct AcknowledgeReadResultView {
    /// Number of messages of the chat still unread by the caller, written by someone else and
    /// posted after the acknowledged message. `0` when everything is read (never `null`), and
    /// always `0` for an administrator who is not a member of the chat.
    #[schema(minimum = 0, example = 2)]
    unread_count: i32,
}

impl AcknowledgeReadResultView {
    pub fn new(unread_count: i32) -> Self {
        Self { unread_count }
    }

    pub fn unread_count(&self) -> i32 {
        self.unread_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_any_positive_bigint() {
        assert!(AcknowledgeReadView::new(1).validate().is_ok());
        assert!(AcknowledgeReadView::new(i64::MAX as u64).validate().is_ok());
    }

    #[test]
    fn rejects_zero_and_values_beyond_bigint() {
        for id in [0, i64::MAX as u64 + 1, u64::MAX] {
            let error = AcknowledgeReadView::new(id).validate().unwrap_err();
            assert!(error.to_string().contains("readUntilMessageId"), "{error}");
        }
    }

    #[test]
    fn body_uses_the_camel_case_field() {
        let view: AcknowledgeReadView =
            serde_json::from_str(r#"{ "readUntilMessageId": 118 }"#).unwrap();
        assert_eq!(view.read_until_message_id(), 118);
        assert!(
            serde_json::from_str::<AcknowledgeReadView>(r#"{ "read_until_message_id": 118 }"#)
                .is_err()
        );
    }
}
