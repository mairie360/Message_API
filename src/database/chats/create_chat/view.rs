use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// Creates a chat and attaches its members in **one statement**: when a member does not exist,
/// the foreign key fails the whole statement and no chat is left behind.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateChatQueryView {
    params: Vec<QueryParam>,
}

impl CreateChatQueryView {
    /// A chat without creator nor member (tests and fixtures).
    pub fn new(title: &str, group_id: Option<i32>) -> Self {
        Self::with_members(title, group_id, None, &[])
    }

    /// A chat created by `created_by` with `members` attached (the creator is not added
    /// implicitly: list it in `members`). `members` must not contain duplicates.
    pub fn with_members(
        title: &str,
        group_id: Option<i32>,
        created_by: Option<u64>,
        members: &[u64],
    ) -> Self {
        // `QueryParam` has no array variant: the ids travel as "1,2,3" and Postgres splits them.
        let csv = members
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        Self {
            params: vec![
                QueryParam::Text(title.to_string()),
                QueryParam::OptionI32(group_id),
                QueryParam::OptionI32(created_by.map(|id| id as i32)),
                QueryParam::Text(csv),
            ],
        }
    }

    pub fn title(&self) -> &str {
        self.params[0].as_text()
    }

    pub fn group_id(&self) -> Option<i32> {
        self.params[1].as_option_i32()
    }

    pub fn created_by(&self) -> Option<i32> {
        self.params[2].as_option_i32()
    }

    pub fn members(&self) -> Vec<i32> {
        let csv = self.params[3].as_text();
        if csv.is_empty() {
            return Vec::new();
        }
        csv.split(',').filter_map(|s| s.parse().ok()).collect()
    }
}

impl Display for CreateChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CreateChatQueryView: title={} group_id={} created_by={} members={:?}",
            self.title(),
            self.group_id().unwrap_or_default(),
            self.created_by().unwrap_or_default(),
            self.members()
        )
    }
}

impl ApiRequestDto for CreateChatQueryView {
    fn query_sql(&self) -> &'static str {
        // `kind` is mandatory since Database 1.2.0: a group chat when group_id is given.
        "WITH chat AS ( \
             INSERT INTO conversations (title, group_id, kind, created_by) \
             VALUES ($1, $2::int, CASE WHEN $2::int IS NULL THEN 'direct' ELSE 'group' END, $3::int) \
             RETURNING id \
         ), members AS ( \
             INSERT INTO conversation_members (conversation_id, user_id) \
             SELECT chat.id, member::integer \
             FROM chat, unnest(string_to_array(NULLIF($4, ''), ',')) AS member \
         ) \
         SELECT id FROM chat"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
