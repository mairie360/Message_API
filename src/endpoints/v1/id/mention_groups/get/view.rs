use utoipa::{IntoParams, ToSchema};

use crate::database::chats::mention_groups::view::MentionGroupRow;
use crate::database::ids::id_from_sql;
use crate::endpoints::validation::{check_opaque, Validate, ValidationError};

/// `groups.name` is `VARCHAR(64)`.
pub const MAX_GROUP_FILTER_LENGTH: usize = 64;

/// Filter of `GET /api/v1/{chat_id}/mention-groups/`.
#[derive(Debug, Default, serde::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MentionGroupsQuery {
    /// Text the group name contains, case-insensitive, at most 64 characters, no control
    /// character. Absent or empty: every group.
    #[param(example = "Urba", max_length = 64)]
    q: Option<String>,
}

impl MentionGroupsQuery {
    pub fn new(q: Option<&str>) -> Self {
        Self {
            q: q.map(str::to_string),
        }
    }

    pub fn q(&self) -> Option<&str> {
        self.q.as_deref()
    }
}

impl Validate for MentionGroupsQuery {
    fn validate(&self) -> Result<(), ValidationError> {
        match &self.q {
            Some(q) => check_opaque("q", q, MAX_GROUP_FILTER_LENGTH),
            None => Ok(()),
        }
    }
}

/// A group that may be mentioned.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
pub struct MentionGroupView {
    /// Core API id of the group, to pass in `group_mentions`.
    #[schema(example = 3)]
    id: u64,
    /// Name of the group.
    #[schema(example = "Urban planning")]
    name: String,
}

impl MentionGroupView {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<MentionGroupRow> for MentionGroupView {
    fn from(row: MentionGroupRow) -> Self {
        Self {
            id: id_from_sql(row.id),
            name: row.name,
        }
    }
}

/// The groups a message of the chat may mention.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ToSchema)]
pub struct MentionGroupsView {
    /// At most 20 groups, by name.
    groups: Vec<MentionGroupView>,
}

impl MentionGroupsView {
    pub fn new(groups: Vec<MentionGroupView>) -> Self {
        Self { groups }
    }

    pub fn groups(&self) -> &[MentionGroupView] {
        &self.groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_bounds() {
        assert!(MentionGroupsQuery::default().validate().is_ok());
        assert!(MentionGroupsQuery::new(Some("")).validate().is_ok());
        assert!(MentionGroupsQuery::new(Some(&"a".repeat(65)))
            .validate()
            .is_err());
        assert!(MentionGroupsQuery::new(Some("a\u{0}")).validate().is_err());
    }
}
