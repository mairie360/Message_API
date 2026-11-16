pub mod error;
pub mod headers;
pub mod health;
pub mod pagination;
pub mod rate_limit;
pub mod ready;
pub mod swagger;
pub mod v1;
pub mod validation;

use actix_web::web;

/// Message of the `400` answered for a path parameter that does not parse. It names the rule,
/// never the value received (MAIR-290).
const INVALID_PATH_MESSAGE: &str = "Invalid path: ids are integers.";

/// A path id that is not an integer answers `400 text/plain`, as the contract documents, instead
/// of actix's default `404` (MAIR-262).
fn path_config() -> web::PathConfig {
    web::PathConfig::default()
        .error_handler(|_, _| actix_web::error::ErrorBadRequest(INVALID_PATH_MESSAGE))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.app_data(path_config()).configure(v1::config);
}
