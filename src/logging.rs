//! Process-wide logger (MAIR-421).
//!
//! Every log line goes through `tracing`: the `log` records of actix and of the libraries are
//! bridged to it. `tracing_actix_web::TracingLogger` (mounted in `main.rs`) opens one span per
//! request carrying its `request_id`, method, route and status, so an error logged by a handler
//! names the request it belongs to.

use tracing_subscriber::EnvFilter;

/// `LOG_FORMAT=text` switches to human-readable lines (dev stack); anything else is JSON, one
/// object per line, for the log collector.
pub const LOG_FORMAT_ENV: &str = "LOG_FORMAT";

/// Level filter used when `RUST_LOG` is not set.
const DEFAULT_FILTER: &str = "info";

/// Installs the global subscriber. Call once, before anything logs.
///
/// # Panics
///
/// Panics if a global subscriber is already installed.
pub fn init() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true);
    let text =
        std::env::var(LOG_FORMAT_ENV).is_ok_and(|format| format.eq_ignore_ascii_case("text"));
    if text {
        builder.init();
    } else {
        builder
            .json()
            .with_current_span(true)
            .flatten_event(true)
            .init();
    }
}
