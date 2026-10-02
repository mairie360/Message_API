use actix_web::{get, HttpResponse, Responder};
use utoipa::OpenApi;

/// Liveness probe: answers as long as the process serves HTTP. Readiness is `GET /ready`.
#[utoipa::path(
    get,
    path = "health",
    summary = "Liveness probe",
    description = "Answers `OK` as soon as the process accepts connections. Unauthenticated, used as the \
                   Kubernetes liveness probe. It deliberately checks neither PostgreSQL nor Redis (a database \
                   outage must not restart every replica): use `GET /ready` to know whether the service can \
                   answer requests.",
    responses(
        (
            status = 200,
            description = "The process accepts connections.",
            body = String,
            content_type = "text/plain",
            example = json!("OK")
        )
    ),
    tag = "Service"
)]
#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[derive(OpenApi)]
#[openapi(paths(health,))]
pub struct HealthDoc;
