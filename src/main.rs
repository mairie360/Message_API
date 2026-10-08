use std::sync::Arc;

use actix_web::{middleware, web, App, HttpServer};

use message_api::database::pg_url::build_pg_url;
use message_api::endpoints::rate_limit::{rate_limit_from_env, rate_limiter};
use message_api::endpoints::swagger::{docs_config, swagger_enabled};
use message_api::endpoints::{config, health, ready};
use message_api::telemetry;

use mairie360_api_lib::env_manager::{get_critical_env_var, get_env_var};
use mairie360_api_lib::security::JwtMiddleware;
use mairie360_api_lib::state::AppState;

use message_api::sse::event_manager::start_internal_event_listener;
use message_api::sse::relay::RedisRelay;

/// Seconds startup waits for PostgreSQL before exiting, unless `DB_STARTUP_TIMEOUT` says otherwise.
const DEFAULT_DB_STARTUP_TIMEOUT: u64 = 60;

//                                        -- MAIN FUNCTION --

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logs on stdout (`src/logging.rs`), plus the trace export when `OTEL_EXPORTER_OTLP_ENDPOINT`
    // is set (MAIR-503); flushed on drop.
    let _telemetry = telemetry::init();
    let redis_url = get_critical_env_var("REDIS_URL");
    let db_user = get_critical_env_var("DB_USER");
    let db_password = get_critical_env_var("DB_PASSWORD");
    let db_host = get_critical_env_var("DB_HOST");
    let db_port = get_critical_env_var("DB_PORT");
    let db_name = get_critical_env_var("DB_NAME");
    let pg_url = build_pg_url(&db_user, &db_password, &db_host, &db_port, &db_name);
    let relay = RedisRelay::new(&redis_url);
    let state = AppState::new(redis_url, pg_url).await;
    // The lib starts without a database (and serves 500s): refuse to start instead, so that the
    // orchestrator restarts the API until Postgres is there.
    let db_startup_timeout = get_env_var("DB_STARTUP_TIMEOUT")
        .and_then(|seconds| seconds.trim().parse().ok())
        .unwrap_or(DEFAULT_DB_STARTUP_TIMEOUT);
    ready::wait_for_postgres(
        state.get_smart_db(),
        std::time::Duration::from_secs(db_startup_timeout),
    )
    .await
    .map_err(|error| {
        tracing::error!("{error}, exiting");
        std::io::Error::other(error)
    })?;
    let host = get_critical_env_var("HOST");
    let port = get_critical_env_var("PORT");
    let bind_address = format!("{}:{}", host, port);

    // 1. Local bus of the chat events (100 buffered events), fed by the write endpoints or, when
    // Redis is reachable, by the relay that shares them between replicas.
    let (bus_tx, _bus_rx) = tokio::sync::broadcast::channel(100);
    if let Some(relay) = &relay {
        tokio::spawn(relay.clone().run(bus_tx.clone()));
    }

    let app_state = Arc::new(message_api::sse::state::AppState::new(bus_tx, relay));

    // 2. SSE listener in the background, next to Actix
    tokio::spawn(start_internal_event_listener(
        app_state.clone(),
        state.get_smart_db().clone(),
    ));
    let data = web::Data::new(state);
    let rate_limit = rate_limit_from_env();
    if rate_limit.is_none() {
        tracing::warn!("RATE_LIMIT_PER_SECOND=0: /api is not rate limited");
    }
    let swagger = swagger_enabled();
    if swagger {
        tracing::warn!("SWAGGER_ENABLED: serving /swagger-ui/ and /api-docs/openapi.json");
    }

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(app_state.clone()))
            .app_data(data.clone())
            // One span per request (request id, method, route, status): the errors logged by the
            // handlers carry it.
            .wrap(tracing_actix_web::TracingLogger::default())
            // Every response is JSON, plain text or an event stream: forbid browsers from sniffing
            // it as HTML.
            .wrap(middleware::DefaultHeaders::new().add(("X-Content-Type-Options", "nosniff")))
            // 1. Swagger UI and the OpenAPI document, only when SWAGGER_ENABLED is set (never in
            // production).
            .configure(|cfg| docs_config(cfg, swagger))
            // 2. Public probes: liveness and readiness
            .service(health::health)
            .service(ready::ready)
            // 3. Every other route requires a JWT, and is rate limited per user (the limiter is
            // wrapped first, so it runs after JwtMiddleware and sees the authenticated user).
            .service(
                web::scope("/api")
                    .wrap(rate_limiter(rate_limit.as_ref()))
                    .wrap(JwtMiddleware)
                    .configure(config),
            )
    })
    .bind(bind_address)?;

    let addr = server.addrs().first().copied();
    tokio::spawn(async move {
        if let Some(addr) = addr {
            tracing::info!("Server listening on http://{addr}");
        }
    });

    server.run().await
}
