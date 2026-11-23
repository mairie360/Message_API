//! `GET /api/v1/ws` (MAIR-657): the WebSocket twin of `GET /api/v1/stream`, following the contract
//! of `docs/en/realtime-websocket.md` (MAIR-656). Both transports share the connection registry,
//! the Redis relay and `event_manager`; only the framing and the supervision loop differ.

use std::sync::Arc;
use std::time::Duration;

use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::{CloseCode, CloseReason, Message, MessageStream, ProtocolError, Session};
use mairie360_api_lib::jwt_manager::{get_jwt_from_request, get_timeout_from_jwt};
use mairie360_api_lib::security::AuthenticatedUser;
use tokio::sync::mpsc;
use tokio::time::Instant;

use crate::sse::state::{AppState, ChatSignal, Connection, Transport};
use crate::sse::supervision::{expiry, still_valid, time_left, PING_INTERVAL, REVALIDATE_INTERVAL};

/// Largest frame accepted from a client, in bytes (`1009` beyond). Clients send no data anyway.
pub const MAX_FRAME_SIZE: usize = 4 * 1024;
/// A connection that answered no ping for this long is closed with `1001`.
pub const PONG_TIMEOUT: Duration = Duration::from_secs(45);
/// Close code: the session expired, was revoked, or the account was removed.
pub const SESSION_ENDED: u16 = 4001;
/// Close code: the same user opened one connection too many, this oldest one is evicted.
pub const EVICTED: u16 = 4008;
/// Signals buffered per connection, as for the SSE: beyond, the socket is closed with `1013`.
const SIGNAL_BUFFER: usize = 10;

/// Intervals of the supervision loop. The route uses [`Timings::default`]; tests shorten them.
#[derive(Debug, Clone, Copy)]
pub struct Timings {
    pub ping_interval: Duration,
    pub pong_timeout: Duration,
    pub revalidate_interval: Duration,
}

impl Default for Timings {
    fn default() -> Self {
        Self {
            ping_interval: PING_INTERVAL,
            pong_timeout: PONG_TIMEOUT,
            revalidate_interval: REVALIDATE_INTERVAL,
        }
    }
}

/// The token a connection was opened with, checked again while it lives.
pub struct Credentials {
    pub user_id: u64,
    pub jwt: String,
    /// Time left before the token's `exp`, `None` when it has no readable expiry.
    pub expires_in: Option<Duration>,
    pub api_state: Arc<mairie360_api_lib::state::AppState>,
}

/// Why the server ended a connection; each cause has its close code (contract of MAIR-656).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseCause {
    /// The client closed the connection or went away: `1000`.
    ClientLeft,
    /// The token expired, its session was revoked or its account removed: `4001`.
    SessionEnded,
    /// A 6th connection of the same user evicted this one: `4008`.
    Evicted,
    /// No pong for [`PONG_TIMEOUT`]: `1001`.
    PongTimeout,
    /// The client sent a data frame, which the channel does not accept: `1003`.
    DataFrame,
    /// The client sent a frame larger than [`MAX_FRAME_SIZE`]: `1009`.
    TooBig,
    /// The client broke the WebSocket protocol: `1002`.
    ProtocolError,
    /// The send buffer was full when a signal came: `1013`, the client reconnects and catches up.
    Overloaded,
}

impl CloseCause {
    /// The close frame sent for this cause.
    pub fn reason(self) -> CloseReason {
        let (code, description) = match self {
            CloseCause::ClientLeft => (CloseCode::Normal, "bye"),
            CloseCause::SessionEnded => (CloseCode::Other(SESSION_ENDED), "session expired"),
            CloseCause::Evicted => (CloseCode::Other(EVICTED), "evicted"),
            CloseCause::PongTimeout => (CloseCode::Away, "pong timeout"),
            CloseCause::DataFrame => (CloseCode::Unsupported, "no data accepted"),
            CloseCause::TooBig => (CloseCode::Size, "frame too large"),
            CloseCause::ProtocolError => (CloseCode::Protocol, "protocol error"),
            CloseCause::Overloaded => (CloseCode::Again, "send buffer full"),
        };
        CloseReason {
            code,
            description: Some(description.to_string()),
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    summary = "Open the real-time notification WebSocket",
    description = "Upgrades the connection to a **WebSocket** the client keeps open to be told about the activity \
                   of its chats, without polling the API (MAIR-657). It carries exactly the signals of \
                   `GET /api/v1/stream`, which stays available until the WebSocket is validated (MAIR-662); the \
                   full contract (frames, close codes, limits, client behaviour) is `docs/en/realtime-websocket.md`.\n\n\
                   Send a regular `GET` with `Upgrade: websocket` and `Authorization: Bearer <JWT>`; nothing goes \
                   in the URL. The token is checked **before** the upgrade: without a valid one the answer is a \
                   `401` and no frame is ever sent.\n\n\
                   **Server → client**: one UTF-8 text frame per `ChatSignal` (JSON, the same object as a `data:` \
                   line of the SSE stream):\n\n\
                   | `type` | When | Fields | Delivered to |\n\
                   | --- | --- | --- | --- |\n\
                   | `NEW_MSG` | a message was posted | `chat_id`, `message_id` | the (not excluded) members but the author |\n\
                   | `MSG_EDITED` | a message was edited | `chat_id`, `message_id` | the members but the editor |\n\
                   | `MSG_DELETED` | a message was deleted (tombstone) | `chat_id`, `message_id` | the members but the deleting user |\n\
                   | `CHAT_READ` | the user acknowledged the chat | `chat_id`, `message_id`, `unread_count` | every connection of that user only |\n\n\
                   Read the change with `GET /api/v1/{chat_id}/?since=<next_since>&since_id=<next_since_id>`; a \
                   signal may arrive twice or be lost, so read since the last cursor after every (re)connection. \
                   Ignore an unknown `type`.\n\n\
                   **Client → server**: no data frame. A text or binary frame closes the connection with `1003`, \
                   a frame larger than 4 KiB with `1009`.\n\n\
                   **Liveness**: the server sends a ping every 15 seconds and closes with `1001` a connection that \
                   answered none for 45 seconds.\n\n\
                   **Server-side closure**: `4001` when the JWT expires, its session is revoked or its account \
                   removed (checked every 30 seconds and at `exp`): renew the session before reconnecting; `4008` \
                   when the same user opens more than 5 connections on this replica, SSE streams included (the \
                   oldest is closed): do not reconnect automatically; `1013` when the server could not keep up \
                   (the 10 signals buffered for this connection were full): reconnect and read since the cursors.",
    responses(
        (
            status = 101,
            description = "Switched to the WebSocket protocol. The body documented is the payload of **one** text \
                           frame, a `ChatSignal`. Example of a `CHAT_READ`: \
                           `{\"type\":\"CHAT_READ\",\"chat_id\":5,\"message_id\":118,\"unread_count\":0}`.",
            body = ChatSignal,
            example = json!({ "type": "NEW_MSG", "chat_id": 5, "message_id": 118 })
        ),
        (
            status = 400,
            description = "Not a WebSocket handshake: `Upgrade: websocket`, `Connection: Upgrade`, \
                           `Sec-WebSocket-Version: 13` or a valid `Sec-WebSocket-Key` is missing.",
            body = String,
            content_type = "text/plain",
            example = json!("")
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, revoked session or unknown \
                           account. The connection is not upgraded.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
        (
            status = 404,
            description = "The token is valid but its account is unknown or archived, as on every route of this API. \
                           The connection is not upgraded.",
            body = String,
            content_type = "text/plain",
            example = json!("Utilisateur inconnu")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Stream",
)]
#[get("/ws")]
async fn websocket_route(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<AppState>,
    api_state: web::Data<mairie360_api_lib::state::AppState>,
    auth_user: AuthenticatedUser,
) -> Result<HttpResponse, actix_web::Error> {
    // The middleware already accepted this token: keep it to check it again while the socket lives.
    let jwt = get_jwt_from_request(&req).unwrap_or_default();
    let expires_in = time_left(get_timeout_from_jwt(&jwt));
    let (response, session, messages) = actix_ws::handle(&req, body)?;

    let (sender, signals) = mpsc::channel(SIGNAL_BUFFER);
    let connection = state.register_as(auth_user.id, sender, Transport::WebSocket);
    let credentials = Credentials {
        user_id: auth_user.id,
        jwt,
        expires_in,
        api_state: api_state.into_inner(),
    };
    actix_web::rt::spawn(serve(
        state.into_inner(),
        connection,
        signals,
        session,
        messages,
        credentials,
        Timings::default(),
    ));
    Ok(response)
}

/// Runs one upgraded connection until it ends: forwards its signals as text frames, pings the
/// client, checks the token, then removes the connection from the registry and closes the socket
/// with the code of the cause, which it returns.
pub async fn serve(
    state: Arc<AppState>,
    connection: Connection,
    mut signals: mpsc::Receiver<Result<web::Bytes, String>>,
    mut session: Session,
    messages: MessageStream,
    credentials: Credentials,
    timings: Timings,
) -> CloseCause {
    let mut messages = messages.max_frame_size(MAX_FRAME_SIZE);
    let mut ping = tokio::time::interval(timings.ping_interval);
    let mut revalidate = tokio::time::interval(timings.revalidate_interval);
    // The first tick of an interval fires immediately, consume it.
    ping.tick().await;
    revalidate.tick().await;
    let expired = expiry(credentials.expires_in);
    tokio::pin!(expired);
    let mut last_pong = Instant::now();

    let cause = loop {
        tokio::select! {
            () = connection.close.notified() => {
                break if connection.overflowed.load(std::sync::atomic::Ordering::Acquire) {
                    CloseCause::Overloaded
                } else {
                    CloseCause::Evicted
                };
            }
            () = &mut expired => break CloseCause::SessionEnded,
            signal = signals.recv() => {
                // The registry keeps a sender as long as the connection is registered, and this
                // function holds one through `connection`: the channel cannot close here.
                let Some(Ok(frame)) = signal else { continue };
                let text = String::from_utf8_lossy(&frame).into_owned();
                if session.text(text).await.is_err() {
                    break CloseCause::ClientLeft;
                }
            }
            message = messages.recv() => match message {
                Some(Ok(Message::Ping(payload))) => {
                    if session.pong(&payload).await.is_err() {
                        break CloseCause::ClientLeft;
                    }
                }
                Some(Ok(Message::Pong(_))) => last_pong = Instant::now(),
                Some(Ok(Message::Nop)) => {}
                Some(Ok(Message::Text(_) | Message::Binary(_) | Message::Continuation(_))) => {
                    break CloseCause::DataFrame;
                }
                Some(Ok(Message::Close(_))) | None => break CloseCause::ClientLeft,
                Some(Err(ProtocolError::Overflow)) => break CloseCause::TooBig,
                Some(Err(_)) => break CloseCause::ProtocolError,
            },
            _ = ping.tick() => {
                if last_pong.elapsed() >= timings.pong_timeout {
                    break CloseCause::PongTimeout;
                }
                if session.ping(b"").await.is_err() {
                    break CloseCause::ClientLeft;
                }
            }
            _ = revalidate.tick() => {
                if !still_valid(&credentials.jwt, &credentials.api_state, credentials.user_id).await {
                    break CloseCause::SessionEnded;
                }
            }
        }
    };

    state.remove_connection(credentials.user_id, connection.id);
    tracing::debug!(user_id = credentials.user_id, cause = ?cause, "WebSocket closed");
    // The client may already be gone: closing a closed session is not an error worth reporting.
    let _ = session.close(Some(cause.reason())).await;
    cause
}
