use crate::endpoints::health::HealthDoc;
use crate::endpoints::ready::ReadyDoc;
use crate::endpoints::v1::doc::V1Doc;
use actix_web::web::ServiceConfig;
use mairie360_api_lib::env_manager::get_env_var;
use utoipa::openapi::schema::{ObjectBuilder, Type};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::openapi::{ContentBuilder, ResponseBuilder};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

/// `SWAGGER_ENABLED=true` (or `1`) serves Swagger UI and `/api-docs/openapi.json`. Unset on the
/// deployed instances (MAIR-424): the contract consumers use is the published
/// `@mairie360/message-api-openapi` package, not the running API. The dev, ZAP and k6 stacks set it.
pub const SWAGGER_ENABLED_ENV: &str = "SWAGGER_ENABLED";

/// Whether [`SWAGGER_ENABLED_ENV`] is enabled.
pub fn swagger_enabled() -> bool {
    get_env_var(SWAGGER_ENABLED_ENV).is_some_and(|value| {
        let value = value.trim();
        value == "1" || value.eq_ignore_ascii_case("true")
    })
}

/// Mounts Swagger UI (`/swagger-ui/`) and the OpenAPI document (`/api-docs/openapi.json`) when
/// `enabled`; mounts nothing otherwise, so both answer `404`.
pub fn docs_config(cfg: &mut ServiceConfig, enabled: bool) {
    if enabled {
        cfg.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Message API — Mairie 360",
        version = "1.0.0",
        description = "\
Messaging API of the **Mairie 360** platform: chats, messages, members and real-time stream. \
Accounts and roles live in Core API.

## Real time

Writes are plain HTTP calls; the other members are told through `GET /api/v1/stream`, a **SSE** \
channel (`text/event-stream`) each client keeps open. Each `data:` line holds a `ChatSignal` saying \
*something happened* in a chat, without the message itself: the client then reloads \
`GET /api/v1/{chat_id}/`.

A `: ping` comment is sent every 15 seconds to keep the connection open through proxies; SSE \
clients ignore it.

## Access control

Every `/api/v1/{chat_id}/**` operation checks the caller against the chat: a caller who is neither \
a member nor an administrator gets the same `404` as for an unknown chat. Members post, edit their \
own messages, leave, and (the creator only) manage the other members; administrators read and \
moderate any chat but do not post in a chat they are not a member of (`403`). Each operation \
describes its own rules.

## Error format

Error responses (`4xx` and `5xx`) have a **`text/plain`** body holding the error message, not a \
JSON object. Every response carries `X-Content-Type-Options: nosniff`.

Statuses returned across the API, before the handler runs:

| Status | Meaning |
| --- | --- |
| `400` | URL segment that is not an integer, malformed JSON body, or a field breaking its \
validation rules (length, control characters); the body names the first invalid field, e.g. \
``Invalid `content`: must be at most 5000 characters``. `<` and `>` are accepted: the API only \
serves JSON, escaping is the job of whoever displays the text. |
| `401` | `Authorization` header missing or malformed, invalid or expired JWT, or revoked session. |
| `429` | Rate limit of the caller exceeded (per user, see `RATE_LIMIT_PER_SECOND` / `RATE_LIMIT_BURST`); \
the body says when to retry. |
| `500` | Database or Redis failure. |
",
        contact(
            name = "Mairie 360 team",
            url = "https://github.com/mairie360"
        ),
        license(
            name = "Proprietary",
            identifier = "LicenseRef-mairie360-proprietary"
        )
    ),
    servers(
        (url = "http://localhost:3003", description = "Local development (cargo run)"),
        (url = "http://development.mairie360.fr", description = "Docker development stack (nginx)")
    ),
    tags(
        (name = "Chats", description = "Chats: list, creation, reading the messages, read acknowledgement and deletion."),
        (name = "Messages", description = "Messages of a chat: posting, editing and deletion."),
        (name = "Users", description = "Members of a chat: listing, adding and removing."),
        (name = "Stream", description = "Real-time SSE notification channel."),
        (name = "Service", description = "Unauthenticated liveness and readiness probes, used by Docker and Kubernetes.")
    ),
    nest(
        (path = "/api/v1", api = V1Doc),
        (path = "/", api = HealthDoc),
        (path = "/", api = ReadyDoc),
    ),
    modifiers(&SecurityAddon, &RateLimitAddon)
)]
pub struct ApiDoc;

/// Without this modifier, the operations declaring `security(("jwt" = []))` reference a scheme
/// missing from the contract: Swagger UI shows no "Authorize" button and generated clients point
/// to nothing.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(
                Http::builder()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT issued by Core API (`POST /api/v1/auth/login`)."))
                    .build(),
            ),
        )
    }
}

/// Adds the `429` of the per-user rate limiter (`rate_limit.rs`, MAIR-425) to every operation that
/// requires a JWT: the limiter answers before any handler runs, so the handlers cannot declare it.
struct RateLimitAddon;

impl Modify for RateLimitAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let too_many_requests = ResponseBuilder::new()
            .description(
                "The caller sent more requests than their rate limit allows (per user, 10 per second \
                 with bursts of 50 by default). Retry after the delay given in the body.",
            )
            .content(
                "text/plain",
                ContentBuilder::new()
                    .schema(Some(ObjectBuilder::new().schema_type(Type::String)))
                    .example(Some(serde_json::json!("Too many requests, retry in 1s")))
                    .build(),
            )
            .build();
        for item in openapi.paths.paths.values_mut() {
            let operations = [
                &mut item.get,
                &mut item.put,
                &mut item.post,
                &mut item.delete,
                &mut item.options,
                &mut item.head,
                &mut item.patch,
                &mut item.trace,
            ];
            for operation in operations.into_iter().flatten() {
                if operation.security.as_ref().is_some_and(|s| !s.is_empty()) {
                    operation
                        .responses
                        .responses
                        .insert("429".to_string(), too_many_requests.clone().into());
                }
            }
        }
    }
}
