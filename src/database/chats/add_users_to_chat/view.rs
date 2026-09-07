use std::fmt::Display;

use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddMembersToChatQueryView {
    params: Vec<QueryParam>,
}

impl AddMembersToChatQueryView {
    pub fn new(chat_id: u64, user_id: Vec<u64>) -> Self {
        // `QueryParam` ne connaît pas les tableaux : on passe la liste des
        // identifiants sous forme de chaîne "1,2,3" que Postgres reconvertit en
        // `integer[]` via `string_to_array`.
        let csv = user_id
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        Self {
            params: vec![QueryParam::I32(chat_id as i32), QueryParam::Text(csv)],
        }
    }

    pub fn chat_id(&self) -> u64 {
        self.params[0].as_i32() as u64
    }

    pub fn user_id(&self) -> Vec<i32> {
        let csv = self.params[1].as_text();
        if csv.is_empty() {
            return Vec::new();
        }
        csv.split(',').filter_map(|s| s.parse().ok()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.params[1].as_text().is_empty()
    }
}

impl Display for AddMembersToChatQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddMembersToChatQueryView: chat_id={} user_id={:?}",
            self.chat_id(),
            self.user_id()
        )
    }
}

impl ApiRequestDto for AddMembersToChatQueryView {
    fn query_sql(&self) -> &'static str {
        // $1 = chat_id, $2 = liste des user_ids au format "1,2,3"
        "INSERT INTO conversation_members (conversation_id, user_id)
         SELECT $1, unnest(string_to_array($2, ','))::integer"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
