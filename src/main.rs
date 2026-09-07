use std::sync::Arc;

use actix_web::{middleware, web, App, HttpServer};

use dashmap::DashMap;
use message_api::endpoints::swagger::ApiDoc;
use message_api::endpoints::{config, health, hello};

use mairie360_api_lib::env_manager::get_critical_env_var;
use mairie360_api_lib::security::JwtMiddleware;
use mairie360_api_lib::state::AppState;

use message_api::sse::event_manager::start_internal_event_listener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

//                                        -- MAIN FUNCTION --

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_url = get_critical_env_var("REDIS_URL");
    let db_user = get_critical_env_var("DB_USER");
    let db_password = get_critical_env_var("DB_PASSWORD");
    let db_host = get_critical_env_var("DB_HOST");
    let db_port = get_critical_env_var("DB_PORT");
    let db_name = get_critical_env_var("DB_NAME");
    let pg_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    );
    let state = AppState::new(redis_url, pg_url).await;
    let host = get_critical_env_var("HOST");
    let port = get_critical_env_var("PORT");
    let bind_address = format!("{}:{}", host, port);

    // 1. Initialisation du canal de broadcast interne (capacité de 100 événements simultanés)
    let (bus_tx, _bus_rx) = tokio::sync::broadcast::channel(100);

    let app_state = Arc::new(message_api::sse::state::AppState {
        online_agents: DashMap::new(),
        internal_bus: bus_tx,
    });

    // 2. On lance l'écouteur SSE en tâche de fond parallèlement à Actix
    tokio::spawn(start_internal_event_listener(
        app_state.clone(),
        state.get_smart_db().clone(),
    ));
    let data = web::Data::new(state);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(app_state.clone()))
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            // 1. Swagger UI et API Docs (Public)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            // 2. Endpoints Publics
            .service(health::health)
            .service(hello::hello)
            // 3. Endpoints Protégés par JWT
            .service(
                web::scope("/api").wrap(JwtMiddleware).configure(config), // Tes routes v1, etc.
            )
    })
    .bind(bind_address)?;

    let addr = server.addrs().first().copied();
    tokio::spawn(async move {
        if let Some(addr) = addr {
            println!("Serveur démarré avec succès sur http://{}", addr);
        }
    });

    server.run().await
}
