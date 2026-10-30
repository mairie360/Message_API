//! OpenTelemetry tracing (MAIR-503, `mairie360_api_lib::logging::telemetry`): each request of
//! this API yields a root span carrying the route and the status, continues an incoming
//! `traceparent`, carries the SQL it ran as span events, and is exported without the client
//! address nor the query string. The redaction of the logs and spans themselves is tested in the
//! lib; the tests below check that this API mounts it (MAIR-290: no query string, no value of a
//! refused body in the request span).

use actix_web::http::Method;
use actix_web::test::TestRequest;
use actix_web::App;
use mairie360_api_lib::logging::telemetry::opentelemetry::{global, Value};
use mairie360_api_lib::logging::telemetry::opentelemetry_sdk::propagation::TraceContextPropagator;
use mairie360_api_lib::logging::telemetry::opentelemetry_sdk::trace::{
    InMemorySpanExporter, SdkTracerProvider, SimpleSpanProcessor, SpanData,
};
use mairie360_api_lib::logging::telemetry::{trace_layer, tracer_provider, TracingLogger};
use mairie360_api_lib::request_log::{hide_query, restore_query, RedactedRootSpanBuilder};
use serial_test::serial;
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::layer::SubscriberExt;

use super::harness::{admin_id, request, states};

const CHATS: &str = "/api/v1/";
const CHATS_ROUTE: &str = "GET /api/v1/";

/// Same mounting as `main.rs`, the redacted `TracingLogger` around the `/api` scope.
macro_rules! traced_app {
    ($state:expr, $sse:expr) => {
        actix_web::test::init_service(
            App::new()
                .app_data($sse.clone())
                .app_data($state.clone())
                .wrap(actix_web::middleware::from_fn(restore_query))
                .wrap(TracingLogger::<RedactedRootSpanBuilder>::new())
                .wrap(actix_web::middleware::from_fn(hide_query))
                .service(
                    actix_web::web::scope("/api")
                        .wrap(mairie360_api_lib::security::JwtMiddleware)
                        .configure(message_api::endpoints::config),
                ),
        )
        .await
    };
}

/// Provider exporting to memory through the redaction of the lib's `tracer_provider`, and the subscriber feeding
/// it, installed for the current thread.
fn in_memory_tracing() -> (InMemorySpanExporter, SdkTracerProvider, DefaultGuard) {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let exporter = InMemorySpanExporter::default();
    let provider = tracer_provider(SimpleSpanProcessor::new(exporter.clone()), "message-api");
    let guard = tracing::subscriber::set_default(
        tracing_subscriber::registry().with(trace_layer(&provider)),
    );
    (exporter, provider, guard)
}

fn request_span(exporter: &InMemorySpanExporter, provider: &SdkTracerProvider) -> SpanData {
    provider.force_flush().expect("flush the spans");
    let spans = exporter.get_finished_spans().unwrap();
    spans
        .iter()
        .find(|span| span.name == CHATS_ROUTE)
        .cloned()
        .unwrap_or_else(|| panic!("no span named {CHATS_ROUTE}: {spans:#?}"))
}

fn attribute<'a>(span: &'a SpanData, key: &str) -> Option<&'a Value> {
    span.attributes
        .iter()
        .find(|kv| kv.key.as_str() == key)
        .map(|kv| &kv.value)
}

#[actix_web::test]
#[serial]
async fn a_request_is_exported_as_a_span_with_its_sql() {
    let (exporter, provider, _guard) = in_memory_tracing();
    let (state, sse) = states().await;
    let admin = admin_id().await;
    let app = traced_app!(state, sse);
    exporter.reset();

    let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
    let request = request(
        Method::GET,
        &format!("{CHATS}?limit=5&offset=0"),
        Some(admin),
    )
    .insert_header(("traceparent", format!("00-{trace_id}-00f067aa0ba902b7-01")))
    .to_request();
    let response = actix_web::test::call_service(&app, request).await;
    assert!(response.status().is_success(), "{}", response.status());
    // The root span closes with the response body: drop it before reading the exported spans.
    drop(response);

    let span = request_span(&exporter, &provider);
    assert_eq!(
        span.span_context.trace_id().to_string(),
        trace_id,
        "the incoming traceparent must be continued"
    );
    assert_eq!(attribute(&span, "http.status_code"), Some(&Value::I64(200)));
    assert_eq!(attribute(&span, "http.route"), Some(&Value::from(CHATS)));
    assert_eq!(
        attribute(&span, "http.target"),
        Some(&Value::from(CHATS)),
        "the query string must not be exported"
    );
    assert_eq!(attribute(&span, "http.client_ip"), None);

    // `sqlx` logs each statement as a `sqlx::query` event; the values stay bound parameters.
    let has_sql = span.events.iter().any(|event| {
        let has = |key: &str| event.attributes.iter().any(|kv| kv.key.as_str() == key);
        has("db.statement")
            && event
                .attributes
                .iter()
                .any(|kv| kv.key.as_str() == "target" && kv.value == Value::from("sqlx::query"))
    });
    assert!(
        has_sql,
        "the SQL run by the handler must be attached to its span: {:#?}",
        span.events
    );
}

#[actix_web::test]
#[serial]
async fn a_refused_request_still_gets_its_span() {
    let (exporter, provider, _guard) = in_memory_tracing();
    let (state, sse) = states().await;
    let app = traced_app!(state, sse);
    exporter.reset();

    // No JWT: `JwtMiddleware` refuses before the handler runs.
    let _ =
        actix_web::test::try_call_service(&app, TestRequest::get().uri(CHATS).to_request()).await;

    let span = request_span(&exporter, &provider);
    assert_eq!(attribute(&span, "http.status_code"), Some(&Value::I64(401)));
}

/// MAIR-290: a refused body is described by its type in the request span, never by the value it
/// held (`TracingLogger::default()` recorded the error `Debug`, where serde quotes the value).
#[actix_web::test]
#[serial]
async fn mair_290_the_span_of_a_refused_body_holds_no_value_of_the_request() {
    const MARKER: &str = "gdpr.marker@example.com";
    let (exporter, provider, _guard) = in_memory_tracing();
    let (state, sse) = states().await;
    let admin = admin_id().await;
    let app = traced_app!(state, sse);
    exporter.reset();

    let request = request(
        Method::POST,
        &format!("{CHATS}?search={MARKER}"),
        Some(admin),
    )
    .set_json(serde_json::json!({ "name": "Astreinte", "members": [MARKER] }))
    .to_request();
    let response = actix_web::test::call_service(&app, request).await;
    assert_eq!(response.status(), 400);
    drop(response);

    provider.force_flush().expect("flush the spans");
    let spans = exporter.get_finished_spans().unwrap();
    let span = spans
        .iter()
        .find(|span| span.name == "POST /api/v1/")
        .unwrap_or_else(|| panic!("no span named POST /api/v1/: {spans:#?}"));
    assert_eq!(attribute(span, "http.target"), Some(&Value::from(CHATS)));
    let details = attribute(span, "exception.details").map(Value::as_str);
    assert!(
        details
            .as_deref()
            .is_some_and(|d| d.starts_with("Json deserialize error")),
        "{details:?}"
    );
    let dump = format!("{:?} {:?}", span.attributes, span.events);
    assert!(!dump.contains("gdpr.marker"), "{dump}");
}
