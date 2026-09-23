pub mod health;
pub mod hello;
pub mod swagger;
pub mod v1;
pub mod validation;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(v1::config);
    cfg.service(health::health);
    cfg.service(hello::hello);
}
