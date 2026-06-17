pub mod doc;
pub mod get;
pub mod id;
pub mod post;
pub mod stream;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/v1"));
}
