//! Mentions and business links of a message (MAIR-1155).
//!
//! The tags travel as one JSON document (`QueryParam` has no array variant), whose keys are the
//! fields the request carried: `mentions` (user ids), `group_mentions` (group ids) and `links`
//! (`[{"kind": "project" | "task" | "event", "id": n}]`). A key absent from the document is left
//! untouched by [`ClearMessageTagsQueryView`], which is how `PATCH` keeps the tags it was not given.

use std::fmt::Display;

use crate::database::ids::{bigint_from_sql, bigint_to_sql, id_from_sql, id_to_sql};
use mairie360_api_lib::database::db_interface::{ApiRequestDto, QueryParam};

/// The first field of the tags document that names something the author may not tag, or `''`:
/// `mentions` (not a non-excluded member of the chat), `group_mentions` (a group none of the
/// non-excluded members belongs to) or `links` (`check_access(author, 'projects' | 'tasks' |
/// 'events', 'read', id)` is not 1: unknown object or no read access).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckMessageTagsQueryView {
    params: Vec<QueryParam>,
}

impl CheckMessageTagsQueryView {
    pub fn new(chat_id: u64, author_id: u64, tags: &serde_json::Value) -> Self {
        Self {
            params: vec![
                QueryParam::I32(id_to_sql(chat_id)),
                QueryParam::I32(id_to_sql(author_id)),
                QueryParam::Text(tags.to_string()),
            ],
        }
    }

    pub fn chat_id(&self) -> u64 {
        id_from_sql(self.params[0].as_i32())
    }

    pub fn author_id(&self) -> u64 {
        id_from_sql(self.params[1].as_i32())
    }
}

impl Display for CheckMessageTagsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CheckMessageTagsQueryView: chat_id={} author_id={}",
            self.chat_id(),
            self.author_id()
        )
    }
}

impl ApiRequestDto for CheckMessageTagsQueryView {
    fn query_sql(&self) -> &'static str {
        // `jsonb_array_elements*` of an absent key yields no row: an absent field is valid.
        "WITH doc AS (SELECT $3::jsonb AS j) \
         SELECT CASE \
             WHEN EXISTS ( \
                 SELECT 1 FROM doc, jsonb_array_elements_text(doc.j -> 'mentions') u(id) \
                 WHERE NOT EXISTS ( \
                     SELECT 1 FROM conversation_members cm \
                     WHERE cm.conversation_id = $1 AND cm.user_id = u.id::int \
                       AND cm.is_excluded = FALSE)) THEN 'mentions' \
             WHEN EXISTS ( \
                 SELECT 1 FROM doc, jsonb_array_elements_text(doc.j -> 'group_mentions') g(id) \
                 WHERE NOT EXISTS ( \
                     SELECT 1 FROM group_members gm \
                     JOIN conversation_members cm ON cm.user_id = gm.user_id \
                         AND cm.conversation_id = $1 AND cm.is_excluded = FALSE \
                     WHERE gm.group_id = g.id::int)) THEN 'group_mentions' \
             WHEN EXISTS ( \
                 SELECT 1 FROM doc, \
                     jsonb_to_recordset(COALESCE(doc.j -> 'links', '[]'::jsonb)) AS l(kind text, id int) \
                 WHERE check_access($2, CASE l.kind WHEN 'project' THEN 'projects' \
                                              WHEN 'task' THEN 'tasks' ELSE 'events' END, \
                                    'read', l.id) <> 1) THEN 'links' \
             ELSE '' END"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Removes the tags of message `message_id` of the kinds present in the document (`mentions`,
/// `group_mentions`, `links`), before [`InsertMessageTagsQueryView`] writes the new ones. Two
/// statements, since a `DELETE` and an `INSERT` of the same row in one statement would both see
/// the old row.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClearMessageTagsQueryView {
    params: Vec<QueryParam>,
}

impl ClearMessageTagsQueryView {
    pub fn new(message_id: u64, tags: &serde_json::Value) -> Self {
        Self {
            params: vec![
                QueryParam::I64(bigint_to_sql(message_id)),
                QueryParam::Text(tags.to_string()),
            ],
        }
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[0].as_i64())
    }
}

impl Display for ClearMessageTagsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClearMessageTagsQueryView: message_id={}",
            self.message_id()
        )
    }
}

impl ApiRequestDto for ClearMessageTagsQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH doc AS (SELECT $2::jsonb AS j), \
         mentions AS ( \
             DELETE FROM message_mentions WHERE message_id = $1 \
               AND (SELECT j ? 'mentions' FROM doc)), \
         group_tags AS ( \
             DELETE FROM message_group_mentions WHERE message_id = $1 \
               AND (SELECT j ? 'group_mentions' FROM doc)) \
         DELETE FROM message_business_links WHERE message_id = $1 \
           AND (SELECT j ? 'links' FROM doc)"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Writes the tags of the document on message `message_id` (duplicates are ignored).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InsertMessageTagsQueryView {
    params: Vec<QueryParam>,
}

impl InsertMessageTagsQueryView {
    pub fn new(message_id: u64, tags: &serde_json::Value) -> Self {
        Self {
            params: vec![
                QueryParam::I64(bigint_to_sql(message_id)),
                QueryParam::Text(tags.to_string()),
            ],
        }
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[0].as_i64())
    }
}

impl Display for InsertMessageTagsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InsertMessageTagsQueryView: message_id={}",
            self.message_id()
        )
    }
}

impl ApiRequestDto for InsertMessageTagsQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH doc AS (SELECT $2::jsonb AS j), \
         mentions AS ( \
             INSERT INTO message_mentions (message_id, user_id) \
             SELECT $1, u.id::int FROM doc, jsonb_array_elements_text(doc.j -> 'mentions') u(id) \
             ON CONFLICT DO NOTHING), \
         group_tags AS ( \
             INSERT INTO message_group_mentions (message_id, group_id) \
             SELECT $1, g.id::int \
             FROM doc, jsonb_array_elements_text(doc.j -> 'group_mentions') g(id) \
             ON CONFLICT DO NOTHING) \
         INSERT INTO message_business_links (message_id, project_id, task_id, event_id) \
         SELECT $1, \
                CASE WHEN l.kind = 'project' THEN l.id END, \
                CASE WHEN l.kind = 'task' THEN l.id END, \
                CASE WHEN l.kind = 'event' THEN l.id END \
         FROM doc, jsonb_to_recordset(COALESCE(doc.j -> 'links', '[]'::jsonb)) AS l(kind text, id int) \
         ON CONFLICT DO NOTHING"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// The members message `message_id` mentions, its author excepted: the non-excluded members of its
/// chat named in `message_mentions`, every one of them when `mentions_everyone`, and those who
/// belong to a group of `message_group_mentions`. The recipients of the `MENTIONED` signal.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MentionRecipientsQueryView {
    params: Vec<QueryParam>,
}

impl MentionRecipientsQueryView {
    pub fn new(message_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I64(bigint_to_sql(message_id))],
        }
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[0].as_i64())
    }
}

impl Display for MentionRecipientsQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MentionRecipientsQueryView: message_id={}",
            self.message_id()
        )
    }
}

impl ApiRequestDto for MentionRecipientsQueryView {
    fn query_sql(&self) -> &'static str {
        "SELECT to_jsonb(cm.user_id) \
         FROM messages m \
         JOIN conversation_members cm ON cm.conversation_id = m.conversation_id \
             AND cm.is_excluded = FALSE AND cm.user_id IS DISTINCT FROM m.owner_id \
         WHERE m.id = $1 AND m.deleted_at IS NULL AND ( \
             m.mentions_everyone \
             OR EXISTS (SELECT 1 FROM message_mentions mm \
                        WHERE mm.message_id = m.id AND mm.user_id = cm.user_id) \
             OR EXISTS (SELECT 1 FROM message_group_mentions g \
                        JOIN group_members gm ON gm.group_id = g.group_id \
                        WHERE g.message_id = m.id AND gm.user_id = cm.user_id)) \
         ORDER BY cm.user_id"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}

/// Removes every reaction, mention and link of message `message_id`: run after it became a
/// tombstone, in the same transaction, as a statement of its own so that it also sees a reaction
/// committed while the deletion waited for the row.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PurgeMessageExtrasQueryView {
    params: Vec<QueryParam>,
}

impl PurgeMessageExtrasQueryView {
    pub fn new(message_id: u64) -> Self {
        Self {
            params: vec![QueryParam::I64(bigint_to_sql(message_id))],
        }
    }

    pub fn message_id(&self) -> u64 {
        bigint_from_sql(self.params[0].as_i64())
    }
}

impl Display for PurgeMessageExtrasQueryView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PurgeMessageExtrasQueryView: message_id={}",
            self.message_id()
        )
    }
}

impl ApiRequestDto for PurgeMessageExtrasQueryView {
    fn query_sql(&self) -> &'static str {
        "WITH reactions AS (DELETE FROM message_reactions WHERE message_id = $1), \
         mentions AS (DELETE FROM message_mentions WHERE message_id = $1), \
         group_tags AS (DELETE FROM message_group_mentions WHERE message_id = $1) \
         DELETE FROM message_business_links WHERE message_id = $1"
    }

    fn query_params(&self) -> &[QueryParam] {
        &self.params
    }
}
