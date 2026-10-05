//! Per-user rate limiting of `/api` (MAIR-425).
//!
//! Every request reaching the API comes from a BFF, so all of them share a handful of IP
//! addresses: the limit is keyed by the authenticated user instead (the middleware runs after
//! `JwtMiddleware`, which stores the `AuthenticatedUser` in the request extensions). A user over
//! the limit gets `429 Too Many Requests` with a `text/plain` body; the other users are not
//! affected.

use std::net::IpAddr;

use actix_governor::{
    governor::middleware::NoOpMiddleware, Governor, GovernorConfig, GovernorConfigBuilder,
    KeyExtractor, SimpleKeyExtractionError,
};
use actix_web::dev::ServiceRequest;
use actix_web::middleware::Condition;
use actix_web::HttpMessage;
use mairie360_api_lib::env_manager::get_env_var;
use mairie360_api_lib::security::AuthenticatedUser;

/// Requests per second refilled for each user; `0` disables the limit.
pub const RATE_LIMIT_PER_SECOND_ENV: &str = "RATE_LIMIT_PER_SECOND";
/// Requests a user may send at once before being limited.
pub const RATE_LIMIT_BURST_ENV: &str = "RATE_LIMIT_BURST";
pub const DEFAULT_PER_SECOND: u64 = 10;
pub const DEFAULT_BURST: u32 = 50;

/// Who a request is counted against.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Caller {
    User(u64),
    /// Only when no user is attached (the limiter is mounted behind `JwtMiddleware`, so this is a
    /// safety net).
    Ip(IpAddr),
}

/// Keys the limit by [`Caller`].
#[derive(Debug, Clone, Copy)]
pub struct CallerKey;

impl KeyExtractor for CallerKey {
    type Key = Caller;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
            return Ok(Caller::User(user.id));
        }
        req.peer_addr()
            .map(|addr| Caller::Ip(addr.ip()))
            .ok_or_else(|| SimpleKeyExtractionError::new("Unable to identify the caller."))
    }
}

pub type RateLimitConfig = GovernorConfig<CallerKey, NoOpMiddleware>;

/// Limit of `per_second` requests per second per caller, with bursts of `burst`; `None` when
/// `per_second` is `0` (no limit).
pub fn rate_limit_config(per_second: u64, burst: u32) -> Option<RateLimitConfig> {
    if per_second == 0 {
        return None;
    }
    GovernorConfigBuilder::default()
        .requests_per_second(per_second)
        .burst_size(burst.max(1))
        .key_extractor(CallerKey)
        .finish()
}

/// The limit configured by [`RATE_LIMIT_PER_SECOND_ENV`] and [`RATE_LIMIT_BURST_ENV`]
/// (defaults: 10 per second, bursts of 50).
pub fn rate_limit_from_env() -> Option<RateLimitConfig> {
    let per_second = get_env_var(RATE_LIMIT_PER_SECOND_ENV)
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(DEFAULT_PER_SECOND);
    let burst = get_env_var(RATE_LIMIT_BURST_ENV)
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(DEFAULT_BURST);
    rate_limit_config(per_second, burst)
}

/// The middleware to wrap the `/api` scope with, *inside* `JwtMiddleware` (wrapped before it):
/// a no-op when `config` is `None`.
///
/// # Panics
///
/// Never: the placeholder quota of the disabled case is valid.
pub fn rate_limiter(
    config: Option<&RateLimitConfig>,
) -> Condition<Governor<CallerKey, NoOpMiddleware>> {
    match config {
        Some(config) => Condition::new(true, Governor::new(config)),
        None => {
            let placeholder = rate_limit_config(1, 1).expect("valid quota");
            Condition::new(false, Governor::new(&placeholder))
        }
    }
}
