//! What keeps a WebSocket connection honest (MAIR-657): the keep-alive interval, the token checks and the timer on the token's expiry.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mairie360_api_lib::jwt_manager::authenticate_token;
use mairie360_api_lib::state::AppState;

/// Interval of the keep-alive ping frame.
pub const PING_INTERVAL: Duration = Duration::from_secs(15);
/// Interval at which the token of an open connection is checked again (expiry, revoked session,
/// deleted account).
pub const REVALIDATE_INTERVAL: Duration = Duration::from_secs(30);

/// Time left before `exp` (seconds since the epoch), `None` when the token carries no readable
/// expiry (it is still checked every [`REVALIDATE_INTERVAL`]).
pub fn time_left(exp: Option<usize>) -> Option<Duration> {
    let exp = Duration::from_secs(u64::try_from(exp?).ok()?);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    Some(exp.saturating_sub(now))
}

/// Sleeps until the token expires, forever when it has no readable expiry.
pub async fn expiry(left: Option<Duration>) {
    match left {
        Some(left) => tokio::time::sleep(left).await,
        None => std::future::pending().await,
    }
}

/// Whether `jwt` still authenticates `user_id`: not expired, session not revoked, account not
/// removed.
pub async fn still_valid(jwt: &str, api_state: &AppState, user_id: u64) -> bool {
    matches!(authenticate_token(jwt, api_state.get_smart_db()).await, Ok(id) if id == user_id)
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
