use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::sse::state::{AppState, ChatSignal};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use mairie360_api_lib::jwt_manager::{
    authenticate_token, get_jwt_from_request, get_timeout_from_jwt,
};
use mairie360_api_lib::security::AuthenticatedUser;
use tokio::sync::mpsc::{self, error::TrySendError};

/// Interval of the `: ping` comment keeping the connection open through proxies.
const PING_INTERVAL: Duration = Duration::from_secs(15);
/// Interval at which the token of an open stream is checked again (expiry, revoked session,
/// deleted account).
const REVALIDATE_INTERVAL: Duration = Duration::from_secs(30);

/// Time left before `exp` (seconds since the epoch), `None` when the token carries no readable
/// expiry (a Keycloak token: it is still checked every [`REVALIDATE_INTERVAL`]).
fn time_left(exp: Option<usize>) -> Option<Duration> {
    let exp = Duration::from_secs(u64::try_from(exp?).ok()?);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    Some(exp.saturating_sub(now))
}

#[utoipa::path(
    get,
    path = "",
    summary = "Open the real-time notification stream",
    description = "Opens an **SSE** channel (`text/event-stream`) the client keeps open to be told about the \
                   activity of its chats, without polling the API.\n\n\
                   Each `data:` line carries a `ChatSignal` saying *that something happened* in a chat; the \
                   message itself is not in it: the client then reloads `GET /api/v1/{chat_id}/`. Only the \
                   (not excluded) members of the chat other than the author are signalled, whichever replica \
                   of the API they are connected to.\n\n\
                   A `: ping` comment is sent every 15 seconds to keep the connection open through proxies; SSE \
                   clients ignore it.\n\n\
                   **The server closes the stream** when the JWT expires, when its session is revoked or its \
                   account removed (checked every 30 seconds), and when the same user opens more than 5 streams \
                   (the oldest one is closed). The client reconnects with a valid token; a closed stream never \
                   ends on its own otherwise.\n\n\
                   Several streams may be open at once for one user (tabs, devices), up to 5: each receives the \
                   signals, and closing one does not affect the others. The `body` documented below describes \
                   the payload of **one** event, not the whole response.",
    responses(
        (
            status = 200,
            description = "SSE stream established. The connection stays open; each `data:` line carries a `ChatSignal`.",
            content_type = "text/event-stream",
            body = ChatSignal,
            example = json!({ "type": "NEW_MSG", "chat_id": 5 })
        ),
        (
            status = 401,
            description = "Missing `Authorization` header, invalid or expired JWT, or revoked session.",
            body = String,
            content_type = "text/plain",
            example = json!("Jeton expiré")
        ),
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Stream",
)]
#[get("/stream")]
async fn sse_stream_route(
    req: HttpRequest,
    state: web::Data<AppState>,
    api_state: web::Data<mairie360_api_lib::state::AppState>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    let user_id = auth_user.id;
    // The middleware already accepted this token: keep it to check it again while the stream lives.
    let jwt = get_jwt_from_request(&req).unwrap_or_default();
    let expires_in = time_left(get_timeout_from_jwt(&jwt));

    // One channel per connection: a user may have several streams open at once.
    let (tx, rx) = mpsc::channel::<Result<actix_web::web::Bytes, String>>(10);
    let connection = state.register(user_id, tx.clone());
    let close = connection.close.clone();

    // Supervisor of this connection: keep-alive ping, token checks, and cleanup once it is gone.
    let state_clone = state.clone();
    actix_web::rt::spawn(async move {
        let mut ping = tokio::time::interval(PING_INTERVAL);
        let mut revalidate = tokio::time::interval(REVALIDATE_INTERVAL);
        // The first tick of an interval fires immediately, consume it.
        ping.tick().await;
        revalidate.tick().await;
        let expiry = async move {
            match expires_in {
                Some(left) => tokio::time::sleep(left).await,
                None => std::future::pending().await,
            }
        };
        tokio::pin!(expiry);

        loop {
            tokio::select! {
                _ = tx.closed() => break,
                _ = &mut expiry => {
                    connection.close.notify_one();
                    break;
                }
                _ = ping.tick() => {
                    let ping_bytes = actix_web::web::Bytes::from(": ping\n\n");
                    // A full buffer is not a dead client: only a closed channel ends the stream.
                    if let Err(TrySendError::Closed(_)) = tx.try_send(Ok(ping_bytes)) {
                        break;
                    }
                }
                _ = revalidate.tick() => {
                    let still_valid = authenticate_token(
                        &jwt,
                        api_state.get_smart_db(),
                        api_state.get_keycloak(),
                    )
                    .await;
                    if !matches!(still_valid, Ok(id) if id == user_id) {
                        connection.close.notify_one();
                        break;
                    }
                }
            }
        }

        state_clone.remove_connection(user_id, connection.id);
    });

    // The response ends when the connection is told to close (eviction, expired or revoked token).
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .take_until(async move { close.notified().await })
        .map(|result| match result {
            Ok(bytes) => Ok::<actix_web::web::Bytes, actix_web::Error>(bytes),
            Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
        });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_left_is_none_without_expiry_and_zero_once_expired() {
        assert_eq!(time_left(None), None);
        assert_eq!(time_left(Some(1)), Some(Duration::ZERO));
        let in_an_hour = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;
        let left = time_left(Some(usize::try_from(in_an_hour).unwrap())).unwrap();
        assert!(left > Duration::from_secs(3590) && left <= Duration::from_secs(3600));
    }
}
