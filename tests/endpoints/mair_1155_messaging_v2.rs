//! MAIR-1155 (messaging V2): reactions, system notes of the member changes, mentions (users,
//! everyone, groups), business links, search in a chat and the groups a message may mention,
//! through the real `/api` scope.

use actix_web::http::{Method, StatusCode};
use mairie360_api_lib::database::db_interface::QueryParam;
use mairie360_api_lib::smart_db::SmartDatabase;
use message_api::sse::state::{ChatEvent, ChatEventKind};
use serde_json::{json, Value};
use tokio::sync::broadcast;

use super::harness::{admin_id, request, states};
use crate::common::{get_fixture_db, named_user, plain_user, unread_count, RawQuery};

/// Every event published on the bus until none comes for half a second.
async fn drain(rx: &mut broadcast::Receiver<ChatEvent>) -> Vec<ChatEvent> {
    let mut events = Vec::new();
    while let Ok(Ok(event)) =
        tokio::time::timeout(std::time::Duration::from_millis(500), rx.recv()).await
    {
        events.push(event);
    }
    events
}

fn of_kind(events: &[ChatEvent], kind: ChatEventKind) -> Vec<&ChatEvent> {
    events.iter().filter(|e| e.kind == kind).collect()
}

/// A group named `name` (unique per run) whose members are `members` and `owner` (Database adds
/// the owner of a group to its members).
async fn group_of(db: &SmartDatabase, owner: u64, name: &str, members: &[u64]) -> u64 {
    let id = db
        .fetch_scalar::<i32, _>(&RawQuery::new(
            "INSERT INTO groups (owner_id, name) VALUES ($1, $2 || ' ' || left(gen_random_uuid()::text, 8)) \
             RETURNING id",
            vec![QueryParam::I32(owner as i32), QueryParam::Text(name.to_string())],
        ))
        .await
        .unwrap();
    for member in members {
        db.execute(RawQuery::new(
            "INSERT INTO group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            vec![QueryParam::I32(id), QueryParam::I32(*member as i32)],
        ))
        .await
        .unwrap();
    }
    id as u64
}

/// A private event created by `owner`.
async fn event_of(db: &SmartDatabase, owner: u64) -> u64 {
    db.fetch_scalar::<i32, _>(&RawQuery::new(
        "INSERT INTO events (name, start_date, end_date, created_by, owner_id) \
         VALUES ('Conseil municipal', now() + interval '1 day', now() + interval '2 days', $1, $1) \
         RETURNING id",
        vec![QueryParam::I32(owner as i32)],
    ))
    .await
    .unwrap() as u64
}

/// A project owned by `owner` and one of its tasks (MAIR-1166). The project gets its template's statuses,
/// and the task the first status of `todo`.
async fn task_of(db: &SmartDatabase, owner: u64) -> (u64, u64) {
    let project = db
        .fetch_scalar::<i32, _>(&RawQuery::new(
            "INSERT INTO projects (title, owner_id) VALUES ('Voirie', $1) RETURNING id",
            vec![QueryParam::I32(owner as i32)],
        ))
        .await
        .unwrap();
    let task = db
        .fetch_scalar::<i32, _>(&RawQuery::new(
            "INSERT INTO tasks (project_id, title) VALUES ($1, 'Rapport') RETURNING id",
            vec![QueryParam::I32(project)],
        ))
        .await
        .unwrap();
    (project as u64, task as u64)
}

macro_rules! chat_of {
    ($app:expr, $creator:expr, $members:expr) => {{
        let (status, body) = send_json!(
            $app,
            request(Method::POST, "/api/v1/", Some($creator))
                .set_json(json!({ "name": "Messagerie V2", "members": $members }))
        );
        assert_eq!(status, StatusCode::OK, "create the chat");
        body["id"].as_u64().unwrap()
    }};
}

macro_rules! post {
    ($app:expr, $chat:expr, $author:expr, $body:expr) => {{
        send_json!(
            $app,
            request(
                Method::POST,
                &format!("/api/v1/{}/messages/", $chat),
                Some($author)
            )
            .set_json($body)
        )
    }};
}

macro_rules! history {
    ($app:expr, $chat:expr, $user:expr) => {{
        let (status, body) = send_json!(
            $app,
            request(Method::GET, &format!("/api/v1/{}/", $chat), Some($user))
        );
        assert_eq!(status, StatusCode::OK);
        body["messages"].as_array().unwrap().clone()
    }};
}

fn message_by_id(messages: &[Value], id: u64) -> Value {
    messages
        .iter()
        .find(|m| m["id"].as_u64() == Some(id))
        .cloned()
        .unwrap_or_else(|| panic!("message {id} in the history"))
}

#[actix_web::test]
async fn mair_1155_mentions_and_links_are_written_read_back_and_signalled() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let admin = admin_id().await;
    let (member, grouped, outsider) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let chat = chat_of!(app, admin, [member, grouped]);
    let group = group_of(&db, outsider, "Urbanisme", &[grouped]).await;
    let empty_group = group_of(&db, outsider, "Vide", &[]).await;
    let event = event_of(&db, admin).await;
    let mut events = sse.internal_bus.subscribe();

    let (status, body) = post!(
        app,
        chat,
        admin,
        json!({
            "content": "@Membre voir l'événement",
            "mentions": [member],
            "group_mentions": [group],
            "links": [{ "kind": "event", "id": event }]
        })
    );
    assert_eq!(status, StatusCode::OK, "{body}");
    let message = body["id"].as_u64().unwrap();

    let read = message_by_id(&history!(app, chat, member), message);
    assert_eq!(read["kind"], "user");
    assert_eq!(read["system_event"], Value::Null);
    assert_eq!(read["mentions"], json!([member]));
    assert_eq!(read["mentions_everyone"], false);
    assert_eq!(read["group_mentions"], json!([group]));
    // MAIR-1166: an event link carries its start, the Calendar front opens it at `?date=<day>&event=<id>`.
    let links = read["links"].as_array().unwrap();
    assert_eq!(links.len(), 1, "{read}");
    assert_eq!(
        (&links[0]["kind"], &links[0]["id"]),
        (&json!("event"), &json!(event))
    );
    assert!(links[0]["start"].as_str().is_some(), "{read}");
    assert!(links[0].get("project_id").is_none(), "{read}");
    assert_eq!(read["reactions"], json!([]));

    let published = drain(&mut events).await;
    let mentioned = of_kind(&published, ChatEventKind::Mentioned);
    assert_eq!(mentioned.len(), 1);
    let mut recipients = mentioned[0].recipients.clone();
    recipients.sort_unstable();
    let mut expected = vec![member, grouped];
    expected.sort_unstable();
    assert_eq!(recipients, expected, "the members named and the group's");
    assert_eq!(mentioned[0].message_id, Some(message));
    assert_eq!(of_kind(&published, ChatEventKind::MessageCreated).len(), 1);

    // @everyone: every member but the author.
    let (status, body) = post!(
        app,
        chat,
        member,
        json!({ "content": "Tout le monde", "mentions_everyone": true })
    );
    assert_eq!(status, StatusCode::OK);
    let published = drain(&mut events).await;
    let mentioned = of_kind(&published, ChatEventKind::Mentioned);
    let mut recipients = mentioned[0].recipients.clone();
    recipients.sort_unstable();
    let mut expected = vec![admin, grouped];
    expected.sort_unstable();
    assert_eq!(recipients, expected);
    let read = message_by_id(&history!(app, chat, admin), body["id"].as_u64().unwrap());
    assert_eq!(read["mentions_everyone"], true);

    // A message without tag publishes no MENTIONED.
    post!(app, chat, member, json!({ "content": "Rien" }));
    let published = drain(&mut events).await;
    assert!(of_kind(&published, ChatEventKind::Mentioned).is_empty());

    // Refusals: nothing is posted.
    let before = history!(app, chat, admin).len();
    for (body, refused) in [
        (
            json!({ "content": "x", "mentions": [outsider] }),
            "`mentions`",
        ),
        (
            json!({ "content": "x", "group_mentions": [empty_group] }),
            "`group_mentions`",
        ),
        (
            json!({ "content": "x", "group_mentions": [2147483000u64] }),
            "`group_mentions`",
        ),
        (
            json!({ "content": "x", "links": [{ "kind": "project", "id": 2147483000u64 }] }),
            "`links`",
        ),
        (
            json!({ "content": "x", "mentions": [member, member] }),
            "`mentions`",
        ),
        (json!({ "content": "x", "mentions": [0] }), "`mentions`"),
        (
            json!({ "content": "x", "links": [{ "kind": "event", "id": event }, { "kind": "event", "id": event }] }),
            "`links`",
        ),
        (
            json!({ "content": "x", "links": [{ "kind": "folder", "id": 1 }] }),
            "",
        ),
    ] {
        let (status, text) = send!(
            app,
            request(
                Method::POST,
                &format!("/api/v1/{chat}/messages/"),
                Some(admin)
            )
            .set_json(&body)
        );
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}: {text}");
        assert!(text.contains(refused), "{body}: {text}");
    }
    // A plain member may not link the administrator's private event.
    let (status, text) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat}/messages/"),
            Some(member)
        )
        .set_json(json!({ "content": "x", "links": [{ "kind": "event", "id": event }] }))
    );
    assert_eq!(status, StatusCode::BAD_REQUEST, "{text}");
    assert!(text.starts_with("`links`"));
    assert_eq!(
        history!(app, chat, admin).len(),
        before,
        "nothing was posted"
    );
}

#[actix_web::test]
async fn mair_1155_patch_replaces_the_tags_it_is_given_and_answers_the_message() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let (author, first, second) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let chat = chat_of!(app, author, [first, second]);
    let (_, body) = post!(
        app,
        chat,
        author,
        json!({ "content": "Bonjour", "mentions": [first] })
    );
    let message = body["id"].as_u64().unwrap();
    let mut events = sse.internal_bus.subscribe();
    let uri = format!("/api/v1/{chat}/messages/{message}/");

    // Absent tags are kept; the answer is the message.
    let (status, body) = send_json!(
        app,
        request(Method::PATCH, &uri, Some(author)).set_json(json!({ "content": "Bonjour !" }))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["content"], "Bonjour !");
    assert_eq!(body["mentions"], json!([first]));
    assert!(body["edited_at"].is_string());
    let published = drain(&mut events).await;
    assert!(of_kind(&published, ChatEventKind::Mentioned).is_empty());

    // Present tags replace: only the newly mentioned member is signalled.
    let (status, body) = send_json!(
        app,
        request(Method::PATCH, &uri, Some(author))
            .set_json(json!({ "content": "Bonjour !", "mentions": [first, second] }))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mentions"], json!([first, second]));
    let published = drain(&mut events).await;
    let mentioned = of_kind(&published, ChatEventKind::Mentioned);
    assert_eq!(mentioned.len(), 1);
    assert_eq!(mentioned[0].recipients, vec![second]);

    // An empty list clears; a refused tag changes nothing.
    let (status, body) = send_json!(
        app,
        request(Method::PATCH, &uri, Some(author))
            .set_json(json!({ "content": "Bonjour", "mentions": [], "mentions_everyone": true }))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mentions"], json!([]));
    assert_eq!(body["mentions_everyone"], true);
    let outsider = plain_user(&db).await;
    let (status, _) = send!(
        app,
        request(Method::PATCH, &uri, Some(author))
            .set_json(json!({ "content": "Changé", "mentions": [outsider] }))
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let read = message_by_id(&history!(app, chat, author), message);
    assert_eq!(read["content"], "Bonjour");
}

#[actix_web::test]
async fn mair_1155_reactions_are_idempotent_per_member_and_emoji() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let admin = admin_id().await;
    let (author, member, outsider) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let chat = chat_of!(app, author, [member]);
    let (_, body) = post!(app, chat, author, json!({ "content": "Réunion à 15h" }));
    let message = body["id"].as_u64().unwrap();
    let (_, header) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat}/"), Some(author))
    );
    let cursor = format!(
        "since={}&since_id={}",
        header["next_since"].as_str().unwrap().replace('+', "%2B"),
        header["next_since_id"]
    );
    let uri = format!("/api/v1/{chat}/messages/{message}/reactions/");
    let mut events = sse.internal_bus.subscribe();

    for (who, emoji) in [
        (member, "👍"),
        (author, "👍"),
        (member, "👍"),
        (member, "🎉"),
    ] {
        let (status, _) = send_json!(
            app,
            request(Method::POST, &uri, Some(who)).set_json(json!({ "emoji": emoji }))
        );
        assert_eq!(status, StatusCode::OK);
    }
    let (_, body) = send_json!(
        app,
        request(Method::POST, &uri, Some(member)).set_json(json!({ "emoji": "🎉" }))
    );
    assert_eq!(
        body["reactions"],
        json!([
            { "emoji": "👍", "count": 2, "user_ids": [member, author] },
            { "emoji": "🎉", "count": 1, "user_ids": [member] }
        ])
    );
    let published = drain(&mut events).await;
    let reacted = of_kind(&published, ChatEventKind::MessageReacted);
    assert_eq!(reacted.len(), 5);
    assert_eq!(reacted[0].message_id, Some(message));

    // The incremental read returns the message with its reactions.
    let (_, changes) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat}/?{cursor}"),
            Some(author)
        )
    );
    let changed = message_by_id(changes["messages"].as_array().unwrap(), message);
    assert_eq!(changed["reactions"][0]["count"], 2);

    // Removal: the caller's own reaction only.
    let delete_uri = format!("{uri}?emoji=%F0%9F%8E%89");
    let (status, body) = send_json!(app, request(Method::DELETE, &delete_uri, Some(member)));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["reactions"].as_array().unwrap().len(), 1);
    let (status, text) = send!(app, request(Method::DELETE, &delete_uri, Some(member)));
    assert_eq!(
        (status, text.as_str()),
        (StatusCode::NOT_FOUND, "Unknown reaction.")
    );
    let (status, _) = send!(app, request(Method::DELETE, &delete_uri, Some(author)));
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Refusals.
    for emoji in ["", "a b", "\u{7}", &"x".repeat(33)] {
        let (status, _) = send!(
            app,
            request(Method::POST, &uri, Some(member)).set_json(json!({ "emoji": emoji }))
        );
        assert_eq!(status, StatusCode::BAD_REQUEST, "{emoji:?}");
    }
    let (status, _) = send!(app, request(Method::DELETE, &uri, Some(member)));
    assert_eq!(status, StatusCode::BAD_REQUEST, "emoji is required");
    let (status, _) = send!(
        app,
        request(Method::POST, &uri, Some(outsider)).set_json(json!({ "emoji": "👍" }))
    );
    assert_eq!(status, StatusCode::NOT_FOUND, "outsider");
    let (status, _) = send!(
        app,
        request(Method::POST, &uri, Some(admin)).set_json(json!({ "emoji": "👍" }))
    );
    assert_eq!(status, StatusCode::FORBIDDEN, "administrator not a member");
    let (status, text) = send!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat}/messages/2147483000/reactions/"),
            Some(member)
        )
        .set_json(json!({ "emoji": "👍" }))
    );
    assert_eq!(
        (status, text.as_str()),
        (StatusCode::NOT_FOUND, "Unknown message.")
    );

    // A deleted message loses its reactions and takes no more.
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat}/messages/{message}/"),
            Some(author)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, changes) = send_json!(
        app,
        request(
            Method::GET,
            &format!("/api/v1/{chat}/?{cursor}"),
            Some(author)
        )
    );
    let tombstone = message_by_id(changes["messages"].as_array().unwrap(), message);
    assert_eq!(tombstone["deleted"], true);
    assert_eq!(tombstone["reactions"], json!([]));
    let (status, _) = send!(
        app,
        request(Method::POST, &uri, Some(member)).set_json(json!({ "emoji": "👍" }))
    );
    assert_eq!(status, StatusCode::NOT_FOUND, "tombstone");
}

#[actix_web::test]
async fn mair_1155_member_changes_write_system_notes() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let (creator, member, joiner, other) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let chat = chat_of!(app, creator, [member]);
    let mut events = sse.internal_bus.subscribe();

    // Addition: the note is the answer and the history's.
    let (status, body) = send_json!(
        app,
        request(
            Method::POST,
            &format!("/api/v1/{chat}/users/"),
            Some(creator)
        )
        .set_json(json!({ "users_id": [joiner, other] }))
    );
    assert_eq!(status, StatusCode::OK);
    let note = &body["system_message"];
    assert_eq!(note["kind"], "system");
    assert_eq!(note["content"], "");
    assert_eq!(note["sender_id"], creator);
    assert_eq!(
        note["system_event"],
        json!({ "action": "members_added", "actor_id": creator, "user_ids": [joiner, other] })
    );
    let note_id = note["id"].as_u64().unwrap();
    let read = message_by_id(&history!(app, chat, joiner), note_id);
    assert_eq!(&read, note);
    // A note is unread for nobody.
    assert_eq!(unread_count(&db, chat, member).await, 0);
    let published = drain(&mut events).await;
    assert_eq!(of_kind(&published, ChatEventKind::MembersChanged).len(), 1);
    let created = of_kind(&published, ChatEventKind::MessageCreated);
    assert_eq!(created[0].message_id, Some(note_id));

    // The members list tells who administers the chat.
    let (_, users) = send_json!(
        app,
        request(Method::GET, &format!("/api/v1/{chat}/users/"), Some(member))
    );
    for user in users["users"].as_array().unwrap() {
        assert_eq!(
            user["is_admin"],
            user["id"].as_u64() == Some(creator),
            "{user}"
        );
    }

    // A system note is neither edited, deleted nor reacted to.
    for (method, uri, body) in [
        (
            Method::PATCH,
            format!("/api/v1/{chat}/messages/{note_id}/"),
            Some(json!({ "content": "Réécrit" })),
        ),
        (
            Method::DELETE,
            format!("/api/v1/{chat}/messages/{note_id}/"),
            None,
        ),
        (
            Method::POST,
            format!("/api/v1/{chat}/messages/{note_id}/reactions/"),
            Some(json!({ "emoji": "👍" })),
        ),
    ] {
        let mut call = request(method.clone(), &uri, Some(creator));
        if let Some(body) = body {
            call = call.set_json(body);
        }
        let (status, text) = send!(app, call);
        assert_eq!(
            (status, text.as_str()),
            (StatusCode::BAD_REQUEST, "A system note cannot be changed."),
            "{method} {uri}"
        );
    }

    // Removal by the creator, then a member leaving.
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat}/users/{other}/"),
            Some(creator)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    let published = drain(&mut events).await;
    let changed = of_kind(&published, ChatEventKind::MembersChanged);
    assert_eq!(
        changed[0].recipients,
        vec![other],
        "the removed member is signalled"
    );
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat}/users/{joiner}/"),
            Some(joiner)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    let notes: Vec<Value> = history!(app, chat, creator)
        .into_iter()
        .filter(|m| m["kind"] == "system")
        .map(|m| m["system_event"].clone())
        .collect();
    assert_eq!(
        notes[1..],
        [
            json!({ "action": "member_removed", "actor_id": creator, "user_ids": [other] }),
            json!({ "action": "member_left", "actor_id": joiner, "user_ids": [joiner] }),
        ]
    );

    // The last members leaving: no note is left behind, the chat is gone.
    for who in [member, creator] {
        let (status, _) = send!(
            app,
            request(
                Method::DELETE,
                &format!("/api/v1/{chat}/users/{who}/"),
                Some(who)
            )
        );
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    let (status, _) = send!(
        app,
        request(Method::GET, &format!("/api/v1/{chat}/"), Some(creator))
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn mair_1155_hiding_a_direct_chat_writes_no_note() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let (alice, bob) = (plain_user(&db).await, plain_user(&db).await);
    let (_, body) = send_json!(
        app,
        request(Method::POST, "/api/v1/direct/", Some(alice))
            .set_json(json!({ "contact_id": bob }))
    );
    let chat = body["id"].as_u64().unwrap();
    post!(app, chat, alice, json!({ "content": "Salut" }));
    let mut events = sse.internal_bus.subscribe();
    let (status, _) = send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat}/users/{alice}/"),
            Some(alice)
        )
    );
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(drain(&mut events).await.is_empty());
    assert!(history!(app, chat, bob).iter().all(|m| m["kind"] == "user"));
}

#[actix_web::test]
async fn mair_1155_search_in_a_chat() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let marker = format!("permis{}", plain_user(&db).await);
    let creator = named_user(&db, "Camille", &format!("Durand{marker}")).await;
    let member = named_user(&db, "Xavier", "Bertrand").await;
    let outsider = plain_user(&db).await;
    let chat = chat_of!(app, creator, [member]);
    let mut ids = Vec::new();
    for content in [
        format!("Le {marker} de la rue Pasteur"),
        "Sans rapport".to_string(),
        format!("Encore le {}", marker.to_uppercase()),
        format!("Supprimé {marker}"),
    ] {
        let (_, body) = post!(app, chat, member, json!({ "content": content }));
        ids.push(body["id"].as_u64().unwrap());
    }
    send!(
        app,
        request(
            Method::DELETE,
            &format!("/api/v1/{chat}/messages/{}/", ids[3]),
            Some(member)
        )
    );
    let search = |query: &str| format!("/api/v1/{chat}/search/?{query}");

    let (status, body) = send_json!(
        app,
        request(Method::GET, &search(&format!("q={marker}")), Some(creator))
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["type"], "messages");
    assert_eq!(body["total"], 2);
    assert_eq!(body["has_more"], false);
    let found: Vec<u64> = body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_u64().unwrap())
        .collect();
    assert_eq!(found, vec![ids[2], ids[0]], "newest first, no tombstone");
    assert_eq!(body["items"][0]["sender_id"], member);

    let (_, body) = send_json!(
        app,
        request(
            Method::GET,
            &search(&format!("q={marker}&limit=1")),
            Some(creator)
        )
    );
    assert_eq!(
        (body["total"].as_u64(), body["has_more"].as_bool()),
        (Some(2), Some(true))
    );
    let (_, body) = send_json!(
        app,
        request(
            Method::GET,
            &search(&format!("q={marker}&limit=1&offset=1")),
            Some(creator)
        )
    );
    assert_eq!(body["items"][0]["id"].as_u64(), Some(ids[0]));
    assert_eq!(body["has_more"], false);

    let (_, body) = send_json!(
        app,
        request(
            Method::GET,
            &search(&format!("q=camille%20durand{marker}&type=people")),
            Some(member)
        )
    );
    assert_eq!(
        body["items"],
        json!([{ "id": creator, "first_name": "Camille", "last_name": format!("Durand{marker}") }])
    );
    let (_, body) = send_json!(
        app,
        request(Method::GET, &search("q=x&type=files"), Some(member))
    );
    assert_eq!(
        body,
        json!({ "type": "files", "items": [], "has_more": false, "total": 0 })
    );
    // `%` only matches itself.
    let (_, body) = send_json!(app, request(Method::GET, &search("q=%25"), Some(member)));
    assert_eq!(body["total"], 0);

    for query in [
        "",
        "q=",
        "q=%20",
        "q=a&limit=51",
        "q=a&type=folders",
        "q=a%07",
    ] {
        let (status, _) = send!(app, request(Method::GET, &search(query), Some(member)));
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}");
    }
    let (status, _) = send!(app, request(Method::GET, &search("q=a"), Some(outsider)));
    assert_eq!(status, StatusCode::NOT_FOUND);
    let admin = admin_id().await;
    let (status, _) = send!(app, request(Method::GET, &search("q=a"), Some(admin)));
    assert_eq!(status, StatusCode::OK, "administrators read any chat");
}

#[actix_web::test]
async fn mair_1155_mention_groups_are_the_groups_of_the_members() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let (creator, member, outsider) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let chat = chat_of!(app, creator, [member]);
    let marker = format!("G{member}");
    let shared = group_of(&db, outsider, &format!("Urbanisme {marker}"), &[member]).await;
    let other = group_of(&db, outsider, &format!("Voirie {marker}"), &[creator]).await;
    group_of(&db, outsider, &format!("Absent {marker}"), &[]).await;
    let uri = format!("/api/v1/{chat}/mention-groups/");

    let (status, body) = send_json!(
        app,
        request(Method::GET, &format!("{uri}?q={marker}"), Some(member))
    );
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<u64> = body["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["id"].as_u64().unwrap())
        .collect();
    assert_eq!(
        ids,
        vec![shared, other],
        "by name, only the members' groups"
    );
    let (_, body) = send_json!(
        app,
        request(
            Method::GET,
            &format!("{uri}?q=voirie%20{marker}"),
            Some(member)
        )
    );
    assert_eq!(body["groups"][0]["id"].as_u64(), Some(other));
    let (status, body) = send_json!(app, request(Method::GET, &uri, Some(member)));
    assert_eq!(status, StatusCode::OK);
    assert!(body["groups"].as_array().unwrap().len() <= 20);

    let (status, _) = send!(
        app,
        request(
            Method::GET,
            &format!("{uri}?q={}", "a".repeat(65)),
            Some(member)
        )
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send!(app, request(Method::GET, &uri, Some(outsider)));
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// MAIR-1166: the links of a message carry what the fronts need to open them, the project of a task and
/// the start of an event, and nothing for a project.
#[actix_web::test]
async fn mair_1166_links_carry_the_project_of_a_task_and_the_start_of_an_event() {
    let (state, sse) = states().await;
    let app = test_app!(state, sse);
    let db = get_fixture_db().await;
    let admin = admin_id().await;
    let member = plain_user(&db).await;
    let chat = chat_of!(app, admin, [member]);
    let (project, task) = task_of(&db, admin).await;
    let event = event_of(&db, admin).await;

    let (status, body) = post!(
        app,
        chat,
        admin,
        json!({
            "content": "voir #Voirie",
            "links": [
                { "kind": "project", "id": project },
                { "kind": "task", "id": task },
                { "kind": "event", "id": event }
            ]
        })
    );
    assert_eq!(status, StatusCode::OK, "{body}");
    let message = body["id"].as_u64().unwrap();

    let read = message_by_id(&history!(app, chat, member), message);
    let links = read["links"].as_array().unwrap();
    assert_eq!(links.len(), 3, "{read}");
    assert_eq!(links[0], json!({ "kind": "project", "id": project }));
    assert_eq!(
        links[1],
        json!({ "kind": "task", "id": task, "project_id": project })
    );
    assert_eq!(
        (&links[2]["kind"], &links[2]["id"]),
        (&json!("event"), &json!(event))
    );
    let start = links[2]["start"].as_str().expect("the start of the event");
    assert!(
        chrono::DateTime::parse_from_rfc3339(start).is_ok(),
        "{start}"
    );
}
