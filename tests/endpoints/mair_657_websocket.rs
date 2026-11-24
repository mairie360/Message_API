//! MAIR-657: `GET /api/v1/ws`, the messaging WebSocket, on a real HTTP server
//! (`actix-test`) with a real WebSocket client (`awc`). Writes go through the HTTP routes, so the
//! signals travel the production path: commit → `AppState::publish` → bus → `event_manager` →
//! registry → socket. The contract checked is `docs/en/realtime-websocket.md` (MAIR-656).

use std::time::Duration;

use actix_codec::Framed;
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse};
use awc::error::WsClientError;
use awc::ws::{CloseCode, Codec, Frame, Message};
use awc::BoxedSocket;
use futures_util::{SinkExt, StreamExt};
use mairie360_api_lib::security::{AuthenticatedUser, JwtMiddleware};
use message_api::endpoints::v1::ws::endpoint::{
    serve, CloseCause, Credentials, Timings, EVICTED, SESSION_ENDED,
};
use message_api::realtime::event_manager::start_internal_event_listener;
use message_api::realtime::state::{AppState as RealtimeState, MAX_CONNECTIONS_PER_USER};
use serde_json::{json, Value};

use super::harness::{core_claims, now, sign_like_core, states, token};
use crate::common::{get_fixture_db, get_smart_db, plain_user};

type Ws = Framed<BoxedSocket, Codec>;

/// How long a socket is watched before concluding that it received nothing.
const SILENCE: Duration = Duration::from_millis(600);

/// Intervals of the `/api/test/ws` route: the production ones would make these tests last minutes.
const FAST: Timings = Timings {
    ping_interval: Duration::from_millis(200),
    pong_timeout: Duration::from_millis(700),
    revalidate_interval: Duration::from_millis(300),
};

/// `GET /api/test/ws`: the production loop (`serve`) with [`FAST`] intervals.
async fn fast_websocket(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<RealtimeState>,
    api_state: web::Data<mairie360_api_lib::state::AppState>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let jwt = mairie360_api_lib::jwt_manager::get_jwt_from_request(&req).unwrap_or_default();
    let (response, session, messages) = actix_ws::handle(&req, body)?;
    let (sender, signals) = tokio::sync::mpsc::channel(10);
    let connection = state.register(user.id, sender);
    let credentials = Credentials {
        user_id: user.id,
        jwt,
        expires_in: None,
        api_state: api_state.into_inner(),
    };
    actix_web::rt::spawn(serve(
        state.into_inner(),
        connection,
        signals,
        session,
        messages,
        credentials,
        FAST,
    ));
    Ok(response)
}

/// The `/api` scope of `main.rs` (plus `/api/test/ws`) on a real server, with the event listener
/// running, as in production without the Redis relay.
async fn server() -> (actix_test::TestServer, web::Data<RealtimeState>) {
    let (state, sse) = states().await;
    actix_web::rt::spawn(start_internal_event_listener(
        sse.clone().into_inner(),
        get_smart_db().await,
    ));
    let app_sse = sse.clone();
    let srv = actix_test::start(move || {
        App::new()
            .app_data(app_sse.clone())
            .app_data(state.clone())
            .service(
                web::scope("/api")
                    .wrap(JwtMiddleware)
                    .route("/test/ws", web::get().to(fast_websocket))
                    .configure(message_api::endpoints::config),
            )
    });
    // Gives the listener time to subscribe to the bus before the first write.
    tokio::time::sleep(Duration::from_millis(200)).await;
    (srv, sse)
}

async fn connect_to(
    srv: &actix_test::TestServer,
    path: &str,
    bearer: Option<&str>,
) -> Result<Ws, WsClientError> {
    let mut request = awc::Client::new().ws(srv.url(path));
    if let Some(bearer) = bearer {
        request = request.bearer_auth(bearer);
    }
    request.connect().await.map(|(_, socket)| socket)
}

async fn connect(srv: &actix_test::TestServer, user: u64) -> Ws {
    connect_to(srv, "/api/v1/ws", Some(&token(user)))
        .await
        .expect("WebSocket upgraded")
}

/// The status of a refused handshake.
fn refused_with(result: Result<Ws, WsClientError>) -> StatusCode {
    match result {
        Err(WsClientError::InvalidResponseStatus(status)) => status,
        Err(other) => panic!("handshake failed without a status: {other}"),
        Ok(_) => panic!("the handshake was accepted"),
    }
}

/// The next frame other than a ping (answered), `None` when nothing came within `within`.
async fn next_frame(ws: &mut Ws, within: Duration) -> Option<Frame> {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        let frame = tokio::time::timeout_at(deadline, ws.next())
            .await
            .ok()??
            .ok()?;
        match frame {
            Frame::Ping(payload) => ws.send(Message::Pong(payload)).await.ok()?,
            frame => return Some(frame),
        }
    }
}

/// The next signal received, as JSON.
async fn next_signal(ws: &mut Ws) -> Value {
    match next_frame(ws, Duration::from_secs(5)).await {
        Some(Frame::Text(text)) => serde_json::from_slice(&text).expect("a JSON text frame"),
        other => panic!("expected a signal, got {other:?}"),
    }
}

/// Whether no signal reached the socket within [`SILENCE`].
async fn silent(ws: &mut Ws) -> bool {
    !matches!(next_frame(ws, SILENCE).await, Some(Frame::Text(_)))
}

/// The close code the server sent, waiting at most `within`.
async fn close_code(ws: &mut Ws, within: Duration) -> Option<u16> {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        match next_frame(ws, left).await? {
            Frame::Close(reason) => return reason.map(|reason| u16::from(reason.code)),
            _ => continue,
        }
    }
}

/// Sends an authenticated JSON request through the test server, returns status and JSON body.
async fn call(
    srv: &actix_test::TestServer,
    method: awc::http::Method,
    path: &str,
    user: u64,
    body: Value,
) -> (StatusCode, Value) {
    let mut response = srv
        .request(method, srv.url(path))
        .bearer_auth(token(user))
        .send_json(&body)
        .await
        .expect("request sent");
    let json = response.json::<Value>().await.unwrap_or_default();
    (response.status(), json)
}

/// A group chat of `creator` with `members`, and its id.
async fn chat_of(srv: &actix_test::TestServer, creator: u64, members: &[u64]) -> u64 {
    let (status, body) = call(
        srv,
        awc::http::Method::POST,
        "/api/v1/",
        creator,
        json!({ "name": "Service état civil", "members": members }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["id"].as_u64().unwrap()
}

async fn post_message(srv: &actix_test::TestServer, chat: u64, author: u64, text: &str) -> u64 {
    let (status, body) = call(
        srv,
        awc::http::Method::POST,
        &format!("/api/v1/{chat}/messages/"),
        author,
        json!({ "content": text }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["id"].as_u64().unwrap()
}

// ----- Handshake (CA1) -----

#[actix_web::test]
async fn mair_657_the_handshake_without_a_token_is_refused_with_401() {
    let (srv, _) = server().await;
    assert_eq!(
        refused_with(connect_to(&srv, "/api/v1/ws", None).await),
        StatusCode::UNAUTHORIZED
    );
}

#[actix_web::test]
async fn mair_657_the_handshake_with_an_invalid_or_expired_token_is_refused_with_401() {
    let (srv, _) = server().await;
    let user = plain_user(&get_fixture_db().await).await;
    let expired = sign_like_core(&core_claims(user, now() - 60));
    for bearer in ["not-a-jwt", expired.as_str()] {
        assert_eq!(
            refused_with(connect_to(&srv, "/api/v1/ws", Some(bearer)).await),
            StatusCode::UNAUTHORIZED,
            "{bearer}"
        );
    }
}

#[actix_web::test]
async fn mair_657_a_plain_get_is_not_upgraded() {
    let (srv, sse) = server().await;
    let user = plain_user(&get_fixture_db().await).await;
    let response = srv
        .get("/api/v1/ws")
        .bearer_auth(token(user))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(sse.online_agents.get(&user).is_none(), "nothing registered");
}

#[actix_web::test]
async fn mair_657_an_upgraded_connection_is_registered() {
    let (srv, sse) = server().await;
    let user = plain_user(&get_fixture_db().await).await;
    let _ws = connect(&srv, user).await;
    assert_eq!(sse.connections_of(user).len(), 1);
}

// ----- Signals and recipients (CA2, CA3, CA4) -----

/// Business journey: A posts, edits then deletes a message in a chat with B; C is an outsider.
/// Only B is told, with the right type and ids; neither the author nor the outsider receive anything.
#[actix_web::test]
async fn mair_657_only_the_other_members_receive_the_message_signals() {
    let (srv, _) = server().await;
    let db = get_fixture_db().await;
    let (a, b, c) = (
        plain_user(&db).await,
        plain_user(&db).await,
        plain_user(&db).await,
    );
    let chat = chat_of(&srv, a, &[a, b]).await;
    let (mut ws_a, mut ws_b, mut ws_c) = (
        connect(&srv, a).await,
        connect(&srv, b).await,
        connect(&srv, c).await,
    );

    let message = post_message(&srv, chat, a, "Le registre est à jour.").await;
    assert_eq!(
        next_signal(&mut ws_b).await,
        json!({ "type": "NEW_MSG", "chat_id": chat, "message_id": message })
    );

    let (status, _) = call(
        &srv,
        awc::http::Method::PATCH,
        &format!("/api/v1/{chat}/messages/{message}/"),
        a,
        json!({ "content": "Le registre est à jour depuis ce matin." }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        next_signal(&mut ws_b).await,
        json!({ "type": "MSG_EDITED", "chat_id": chat, "message_id": message })
    );

    let (status, _) = call(
        &srv,
        awc::http::Method::DELETE,
        &format!("/api/v1/{chat}/messages/{message}/"),
        a,
        json!({}),
    )
    .await;
    assert!(status.is_success(), "{status}");
    assert_eq!(
        next_signal(&mut ws_b).await,
        json!({ "type": "MSG_DELETED", "chat_id": chat, "message_id": message })
    );

    assert!(silent(&mut ws_a).await, "the author is never signalled");
    assert!(silent(&mut ws_c).await, "an outsider receives nothing");
}

/// A acknowledges the chat on one tab: every connection of A (the other tab included) gets
/// `CHAT_READ` with the unread count, B gets nothing.
#[actix_web::test]
async fn mair_657_a_read_acknowledgement_reaches_the_readers_connections_only() {
    let (srv, _) = server().await;
    let db = get_fixture_db().await;
    let (a, b) = (plain_user(&db).await, plain_user(&db).await);
    let chat = chat_of(&srv, a, &[a, b]).await;
    let (mut tab_1, mut tab_2, mut ws_b) = (
        connect(&srv, a).await,
        connect(&srv, a).await,
        connect(&srv, b).await,
    );
    let message = post_message(&srv, chat, b, "Pouvez-vous valider ?").await;
    for tab in [&mut tab_1, &mut tab_2] {
        assert_eq!(next_signal(tab).await["type"], "NEW_MSG");
    }

    let (status, body) = call(
        &srv,
        awc::http::Method::POST,
        &format!("/api/v1/{chat}/read/"),
        a,
        json!({ "readUntilMessageId": message }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let expected =
        json!({ "type": "CHAT_READ", "chat_id": chat, "message_id": message, "unread_count": 0 });
    assert_eq!(next_signal(&mut tab_1).await, expected);
    assert_eq!(next_signal(&mut tab_2).await, expected);
    assert!(
        silent(&mut ws_b).await,
        "nobody else is told about A's read"
    );
}

/// Two open connections of one member (two tabs) receive the same signal, each as the bare JSON
/// text frame.
#[actix_web::test]
async fn mair_662_two_connections_of_a_member_receive_the_same_bare_frame() {
    let (srv, sse) = server().await;
    let db = get_fixture_db().await;
    let (a, b) = (plain_user(&db).await, plain_user(&db).await);
    let chat = chat_of(&srv, a, &[a, b]).await;
    let mut ws_b = connect(&srv, b).await;
    let (sender, mut other_tab) = tokio::sync::mpsc::channel(4);
    sse.register(b, sender);

    let message = post_message(&srv, chat, a, "Les deux onglets.").await;
    let signal = json!({ "type": "NEW_MSG", "chat_id": chat, "message_id": message });
    assert_eq!(next_signal(&mut ws_b).await, signal);
    let frame = tokio::time::timeout(Duration::from_secs(5), other_tab.recv())
        .await
        .expect("signal in time")
        .expect("channel open")
        .unwrap();
    assert_eq!(serde_json::from_slice::<Value>(&frame).unwrap(), signal);
}

// ----- Limits and closure (CA6, CA7, contract) -----

/// The 6th connection of a user closes the oldest with `4008`; the 5 others keep receiving.
#[actix_web::test]
async fn mair_657_a_sixth_connection_evicts_the_oldest_with_4008() {
    let (srv, sse) = server().await;
    let db = get_fixture_db().await;
    let (a, b) = (plain_user(&db).await, plain_user(&db).await);
    let chat = chat_of(&srv, a, &[a, b]).await;

    let mut oldest = connect(&srv, b).await;
    let mut others = Vec::new();
    for _ in 0..MAX_CONNECTIONS_PER_USER {
        others.push(connect(&srv, b).await);
    }
    assert_eq!(
        close_code(&mut oldest, Duration::from_secs(5)).await,
        Some(EVICTED)
    );
    assert_eq!(sse.connections_of(b).len(), MAX_CONNECTIONS_PER_USER);

    let message = post_message(&srv, chat, a, "Toujours reçu.").await;
    for ws in &mut others {
        assert_eq!(next_signal(ws).await["message_id"], message);
    }
}

/// The socket is closed with `4001` when the token it was opened with expires, without waiting for
/// the next 30 s revalidation.
#[actix_web::test]
async fn mair_657_the_socket_closes_with_4001_when_its_jwt_expires() {
    let (srv, sse) = server().await;
    let user = plain_user(&get_fixture_db().await).await;
    let short = sign_like_core(&core_claims(user, now() + 2));
    let mut ws = connect_to(&srv, "/api/v1/ws", Some(&short)).await.unwrap();

    assert_eq!(
        close_code(&mut ws, Duration::from_secs(6)).await,
        Some(SESSION_ENDED)
    );
    assert!(
        sse.online_agents.get(&user).is_none(),
        "removed from the registry"
    );
}

/// An account archived while its socket is open is closed with `4001` at the next revalidation.
#[actix_web::test]
async fn mair_657_the_socket_closes_with_4001_when_its_account_is_archived() {
    let (srv, _) = server().await;
    let db = get_fixture_db().await;
    let user = plain_user(&db).await;
    let mut ws = connect_to(&srv, "/api/test/ws", Some(&token(user)))
        .await
        .unwrap();

    db.execute(ArchiveUser {
        params: vec![mairie360_api_lib::database::db_interface::QueryParam::I32(
            i32::try_from(user).unwrap(),
        )],
    })
    .await
    .unwrap();
    assert_eq!(
        close_code(&mut ws, Duration::from_secs(3)).await,
        Some(SESSION_ENDED)
    );
}

#[actix_web::test]
async fn mair_657_a_client_data_frame_closes_with_1003() {
    let (srv, _) = server().await;
    let user = plain_user(&get_fixture_db().await).await;
    let mut ws = connect(&srv, user).await;
    ws.send(Message::Text("hello".into())).await.unwrap();
    assert_eq!(
        close_code(&mut ws, Duration::from_secs(3)).await,
        Some(u16::from(CloseCode::Unsupported))
    );
}

#[actix_web::test]
async fn mair_657_a_frame_over_4_kib_closes_with_1009() {
    let (srv, _) = server().await;
    let user = plain_user(&get_fixture_db().await).await;
    let mut ws = connect(&srv, user).await;
    ws.send(Message::Binary(vec![0_u8; 5 * 1024].into()))
        .await
        .unwrap();
    assert_eq!(
        close_code(&mut ws, Duration::from_secs(3)).await,
        Some(u16::from(CloseCode::Size))
    );
}

// ----- Liveness (contract) -----

/// A client answering the pings stays open past the pong timeout.
#[actix_web::test]
async fn mair_657_a_client_answering_pings_stays_open() {
    let (srv, sse) = server().await;
    let user = plain_user(&get_fixture_db().await).await;
    let mut ws = connect_to(&srv, "/api/test/ws", Some(&token(user)))
        .await
        .unwrap();
    // `next_frame` answers every ping; nothing else may arrive for three pong timeouts.
    assert!(next_frame(&mut ws, FAST.pong_timeout * 3).await.is_none());
    assert_eq!(sse.connections_of(user).len(), 1);
}

/// A client that answers no ping is closed with `1001` and removed from the registry.
#[actix_web::test]
async fn mair_657_a_client_answering_no_ping_is_closed_with_1001() {
    let (srv, sse) = server().await;
    let user = plain_user(&get_fixture_db().await).await;
    let mut ws = connect_to(&srv, "/api/test/ws", Some(&token(user)))
        .await
        .unwrap();

    let deadline = tokio::time::Instant::now() + FAST.pong_timeout * 4;
    let mut code = None;
    let mut pinged = false;
    // Read the frames without answering the pings.
    while let Ok(Some(Ok(frame))) = tokio::time::timeout_at(deadline, ws.next()).await {
        match frame {
            Frame::Ping(_) => pinged = true,
            Frame::Close(reason) => {
                code = reason.map(|reason| u16::from(reason.code));
                break;
            }
            _ => {}
        }
    }
    assert!(pinged, "the server pings");
    assert_eq!(code, Some(u16::from(CloseCode::Away)));
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        sse.online_agents.get(&user).is_none(),
        "removed from the registry"
    );
}

#[test]
fn mair_657_close_causes_map_to_the_contract_codes() {
    let code = |cause: CloseCause| u16::from(cause.reason().code);
    assert_eq!(code(CloseCause::ClientLeft), 1000);
    assert_eq!(code(CloseCause::PongTimeout), 1001);
    assert_eq!(code(CloseCause::ProtocolError), 1002);
    assert_eq!(code(CloseCause::DataFrame), 1003);
    assert_eq!(code(CloseCause::TooBig), 1009);
    assert_eq!(code(CloseCause::SessionEnded), 4001);
    assert_eq!(code(CloseCause::Evicted), 4008);
    assert_eq!(code(CloseCause::Overloaded), 1013);
}

/// A socket whose send buffer overflowed is closed with `1013` (its client reconnects and catches
/// up) and leaves the registry; the other connections of the user stay open.
#[actix_web::test]
async fn mair_657_an_overflowing_socket_is_closed_with_1013() {
    let (srv, sse) = server().await;
    let db = get_fixture_db().await;
    let user = plain_user(&db).await;
    let mut overflowing = connect(&srv, user).await;
    let mut other = connect(&srv, user).await;

    // What `AppState::deliver` does when the buffer of this socket is full.
    let first = sse.online_agents.get(&user).unwrap()[0].clone();
    first
        .overflowed
        .store(true, std::sync::atomic::Ordering::Release);
    first.close.notify_one();

    assert_eq!(
        close_code(&mut overflowing, Duration::from_secs(3)).await,
        Some(1013)
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    let left: Vec<u64> = sse.connections_of(user).iter().map(|(id, _)| *id).collect();
    assert_eq!(left.len(), 1, "only the overflowing socket left");
    assert!(!left.contains(&first.id));
    assert!(silent(&mut other).await, "the other socket stays open");
}

/// Archives an account, as Core does when it is removed.
#[derive(serde::Serialize, serde::Deserialize)]
struct ArchiveUser {
    params: Vec<mairie360_api_lib::database::db_interface::QueryParam>,
}

impl std::fmt::Display for ArchiveUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArchiveUser")
    }
}

impl mairie360_api_lib::database::db_interface::ApiRequestDto for ArchiveUser {
    fn query_sql(&self) -> &'static str {
        "UPDATE users SET is_archived = TRUE, status = 'archived' WHERE id = $1"
    }

    fn query_params(&self) -> &[mairie360_api_lib::database::db_interface::QueryParam] {
        &self.params
    }
}
