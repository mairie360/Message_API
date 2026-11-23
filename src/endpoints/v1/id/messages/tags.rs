//! Mentions and business links carried by a message (MAIR-1155), shared by `POST` and `PATCH`.
//!
//! In a `POST`, an absent field means none. In a `PATCH`, an absent field keeps what the message
//! has, a present one (even empty) replaces it.

use std::collections::HashSet;

use mairie360_api_lib::smart_db::SmartTransaction;

use crate::database::chats::message_tags::view::{
    CheckMessageTagsQueryView, ClearMessageTagsQueryView, InsertMessageTagsQueryView,
    MentionRecipientsQueryView,
};
use crate::database::ids::{id_from_sql, id_to_sql};
use crate::endpoints::error::unexpected;
use crate::endpoints::v1::id::get::view::MessageLinkView;
use crate::endpoints::validation::{check_ids, ValidationError, MAX_USER_ID};

/// Most entries of each list: `mentions`, `group_mentions`, `links`.
pub const MAX_TAGS: usize = 50;

/// The tags of a message request, each `None` when the request did not carry it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageTags {
    pub mentions: Option<Vec<u64>>,
    pub mentions_everyone: Option<bool>,
    pub group_mentions: Option<Vec<u64>>,
    pub links: Option<Vec<MessageLinkView>>,
}

impl MessageTags {
    /// Shape rules: at most [`MAX_TAGS`] of each, ids 1..=2147483647, no duplicate.
    ///
    /// # Errors
    ///
    /// The first field breaking a rule.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(ids) = &self.mentions {
            check_ids("mentions", ids, MAX_TAGS, true, "users")?;
        }
        if let Some(ids) = &self.group_mentions {
            check_ids("group_mentions", ids, MAX_TAGS, true, "groups")?;
        }
        if let Some(links) = &self.links {
            if links.len() > MAX_TAGS {
                return Err(ValidationError::new(
                    "links",
                    &format!("must hold at most {MAX_TAGS} links"),
                ));
            }
            if links.iter().any(|l| l.id() == 0 || l.id() > MAX_USER_ID) {
                return Err(ValidationError::new(
                    "links",
                    &format!("ids must be between 1 and {MAX_USER_ID}"),
                ));
            }
            let unique: HashSet<&MessageLinkView> = links.iter().collect();
            if unique.len() != links.len() {
                return Err(ValidationError::new("links", "must not contain duplicates"));
            }
        }
        Ok(())
    }

    /// Whether the request carries a list to check or write.
    pub fn has_lists(&self) -> bool {
        self.mentions.is_some() || self.group_mentions.is_some() || self.links.is_some()
    }

    /// The document the database views read: one key per list present.
    pub fn to_document(&self) -> serde_json::Value {
        let mut document = serde_json::Map::new();
        if let Some(ids) = &self.mentions {
            document.insert("mentions".into(), ids_json(ids));
        }
        if let Some(ids) = &self.group_mentions {
            document.insert("group_mentions".into(), ids_json(ids));
        }
        if let Some(links) = &self.links {
            document.insert(
                "links".into(),
                links
                    .iter()
                    .map(|l| serde_json::json!({ "kind": l.kind().as_str(), "id": id_to_sql(l.id()) }))
                    .collect(),
            );
        }
        serde_json::Value::Object(document)
    }
}

fn ids_json(ids: &[u64]) -> serde_json::Value {
    ids.iter().map(|id| id_to_sql(*id)).collect()
}

/// Body of the `400` answered when a tag names something the author may not tag.
pub fn refused_tag_message(field: &str) -> &'static str {
    match field {
        "mentions" => "`mentions` contains a user who is not a member of this chat.",
        "group_mentions" => "`group_mentions` contains a group no member of this chat belongs to.",
        _ => "`links` contains an object that does not exist or that you may not read.",
    }
}

/// Why the tags of a message were not written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagsFailure {
    /// A tag names something the author may not tag: the body of the `400`.
    Refused(&'static str),
    DatabaseError,
}

/// Checks, inside the transaction of the write, that every member mentioned is a non-excluded
/// member of chat `chat_id`, every group mentioned has such a member and every object linked is
/// readable by `author_id` (`check_access(.., 'read', id) = 1`).
pub async fn check_tags(
    tx: &mut SmartTransaction,
    chat_id: u64,
    author_id: u64,
    tags: &MessageTags,
) -> Result<(), TagsFailure> {
    if !tags.has_lists() {
        return Ok(());
    }
    let refused: String = tx
        .fetch_scalar(&CheckMessageTagsQueryView::new(
            chat_id,
            author_id,
            &tags.to_document(),
        ))
        .await
        .map_err(|e| {
            unexpected("check message tags", e);
            TagsFailure::DatabaseError
        })?;
    if refused.is_empty() {
        Ok(())
    } else {
        Err(TagsFailure::Refused(refused_tag_message(&refused)))
    }
}

/// Writes the lists of `tags` on message `message_id`, replacing those of the same kinds.
pub async fn write_tags(
    tx: &mut SmartTransaction,
    message_id: u64,
    tags: &MessageTags,
) -> Result<(), TagsFailure> {
    if !tags.has_lists() {
        return Ok(());
    }
    let document = tags.to_document();
    tx.execute(&ClearMessageTagsQueryView::new(message_id, &document))
        .await
        .map_err(|e| {
            unexpected("clear message tags", e);
            TagsFailure::DatabaseError
        })?;
    tx.execute(&InsertMessageTagsQueryView::new(message_id, &document))
        .await
        .map_err(|e| {
            unexpected("insert message tags", e);
            TagsFailure::DatabaseError
        })
}

/// The members message `message_id` mentions (users, everyone, groups), its author excepted.
pub async fn mention_recipients(
    tx: &mut SmartTransaction,
    message_id: u64,
) -> Result<Vec<u64>, TagsFailure> {
    let ids: Vec<i32> = tx
        .fetch_all(&MentionRecipientsQueryView::new(message_id))
        .await
        .map_err(|e| {
            unexpected("mention recipients", e);
            TagsFailure::DatabaseError
        })?;
    Ok(ids.into_iter().map(id_from_sql).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoints::v1::id::get::view::LinkKind;

    #[test]
    fn the_document_only_holds_the_lists_present() {
        let tags = MessageTags {
            mentions: Some(vec![42]),
            mentions_everyone: Some(true),
            group_mentions: None,
            links: Some(vec![MessageLinkView::new(LinkKind::Event, 9)]),
        };
        assert_eq!(
            tags.to_document(),
            serde_json::json!({ "mentions": [42], "links": [{ "kind": "event", "id": 9 }] })
        );
        assert!(tags.has_lists());
        assert!(!MessageTags::default().has_lists());
    }

    #[test]
    fn shape_rules() {
        assert!(MessageTags::default().validate().is_ok());
        let duplicate_links = MessageTags {
            links: Some(vec![
                MessageLinkView::new(LinkKind::Project, 1),
                MessageLinkView::new(LinkKind::Project, 1),
            ]),
            ..MessageTags::default()
        };
        assert!(duplicate_links.validate().is_err());
        let same_id_other_kind = MessageTags {
            links: Some(vec![
                MessageLinkView::new(LinkKind::Project, 1),
                MessageLinkView::new(LinkKind::Task, 1),
            ]),
            ..MessageTags::default()
        };
        assert!(same_id_other_kind.validate().is_ok());
        let zero = MessageTags {
            links: Some(vec![MessageLinkView::new(LinkKind::Task, 0)]),
            ..MessageTags::default()
        };
        assert!(zero.validate().is_err());
        let too_many = MessageTags {
            mentions: Some((1..=51).collect()),
            ..MessageTags::default()
        };
        assert!(too_many.validate().is_err());
        assert!(refused_tag_message("links").starts_with("`links`"));
    }
}
