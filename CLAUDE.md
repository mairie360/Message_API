# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`message_api` is the chat/messaging microservice of the **mairie360** platform: an Actix-web
(Rust) HTTP API backed by PostgreSQL and Redis, with a Server-Sent-Events channel for
real-time message notifications. It was bootstrapped from a "Rust API Template" (see
`README.md`); leftover `#change api name` / `#change port` markers in `Cargo.toml`,
`docker-compose.yml` and `nginx.conf` are template residue, not TODOs for a fresh rename.

Most cross-cutting behavior (DB + cache access, auth, env access, query trait, test harness)
lives in the external crate **`mairie360_api_lib`** (crates.io, `1.2.0`, pinned in
`Cargo.lock`). Read its source under `~/.cargo/registry/src/*/mairie360_api_lib-1.2.0/`
when in doubt — the API surface used here is: `state::AppState`
(`get_smart_db()` / `get_redis()`), `smart_db::SmartDatabase`
(`fetch_one` / `fetch_all` / `fetch_scalar` / `execute`),
`database::db_interface::{ApiRequestDto, QueryParam}`,
`database::error::DbError`, `error::ApiLibError`,
`security::{JwtMiddleware, AuthenticatedUser}`, `env_manager::get_critical_env_var`,
`test_setup::queries_setup::get_shared_db`. This crate owns `sqlx` — the API itself has
no direct `sqlx` dependency.

## Commands

Cargo aliases are defined in `.cargo/config.toml`:

| Task | Command |
| --- | --- |
| Build | `cargo build` |
| Format check (CI gate) | `cargo lint_check` (`fmt --all -- --check`) |
| Format fix | `cargo lint_fix` |
| Clippy (CI gate, warnings = errors) | `cargo check_code` (`clippy --all-targets --all-features -- -D warnings`) |
| Regenerate OpenAPI spec | `cargo open_api > openapi.json` |
| Regenerate the TS client | `npx orval` (reads `openapi.json` → `generated/`) |
| Run locally | needs all env vars set (see below), then `cargo run` |
| Full dev stack + hot reload | `docker compose up --watch` |

### Tests

Integration tests live in `tests/` (there is no meaningful unit-test suite in `src/`).
They are plain `#[tokio::test]` + `#[serial]` (`serial_test`) and use `mairie360_api_lib`'s
`get_shared_db()`, which spins up **real Docker containers via testcontainers** —
`ghcr.io/mairie360/database:1.0.0` plus a Liquibase migration container, in `--network host`
mode (binds `127.0.0.1:5432`, so nothing else may hold that port, and tests must stay
`#[serial]`). A running Docker daemon and pull access to `ghcr.io/mairie360/*` are required.

`tests/common::get_smart_db(db_url)` builds a `SmartDatabase` over that Postgres. It also
constructs a `Redis` pointing at `localhost:6379`, but no chat `QueryView` declares a
`cache_key`, so Redis is never actually contacted — tests do not need a Redis container.

```bash
cargo test                                 # whole suite
cargo test --test integration_test         # the one integration binary
cargo test test_create_chat_success        # a single test by name
```

The shared container/seed data is initialized once per run (`OnceCell`), so tests share
one DB and must not depend on a pristine schema.

## Architecture

### Routing mirrors the URL tree on the filesystem

Under `src/endpoints/`, every URL path segment is a module directory whose `mod.rs`
exposes `pub fn config(cfg: &mut ServiceConfig)` and builds one `web::scope(...)`, then
`.configure(child::config)` for nested segments. Path params get their own directory:
`{chat_id}` → `v1/id/`, `{message_id}` → `v1/id/messages/id/`. Within a leaf endpoint dir:

- `endpoint.rs` — the handler (`#[get]`/`#[post]`/…), a local `enum XxxError` implementing
  `Display` + `actix_web::ResponseError`, a private `trigger_*` async fn holding the real
  logic, and the `#[utoipa::path(...)]` annotation.
- `view.rs` — request/response DTOs (`serde` + `utoipa::ToSchema`); request DTOs get a
  `TryFrom<web::Json<Self>>` used by the handler to map deserialization into `BadRequest`.
- `doc.rs` — a utoipa `OpenApi` struct that lists this level's `paths(...)` /
  `components(schemas(...))` and `nest(...)`s the children's `*Doc` structs.

`src/endpoints/swagger.rs::ApiDoc` is the root of that `doc.rs` nesting tree. Keep the
three in sync when adding an endpoint: register it in the parent `config()`, add its schemas
and `__path_*` to the parent `doc.rs`, then regenerate `openapi.json`.

`src/main.rs` mounts Swagger UI + `/health` + `/hello` publicly and everything else under
`web::scope("/api").wrap(JwtMiddleware)`. `JwtMiddleware` (from the lib) additionally
whitelists `/`, `/swagger-ui*`, `/api-docs*`, and any path containing `/auth`. On success it
inserts an `AuthenticatedUser { id }` into request extensions; handlers get it via the
`AuthenticatedUser` extractor argument.

### Two `AppState`s coexist as Actix `web::Data`

- `mairie360_api_lib::state::AppState` — wraps a `SmartDatabase` (`get_smart_db()`) and a
  `Redis` (`get_redis()`). `SmartDatabase` is cache-aside: `Redis` / `Database` connection
  failures are swallowed at startup and degrade to direct Postgres, so handlers get a
  `&SmartDatabase` unconditionally (no `Option`).
- `message_api::sse::state::AppState` — SSE runtime state (see below), wrapped in an `Arc`
  and registered with `web::Data::from`.

Handlers that need both take two `web::Data<...>` arguments (the module paths disambiguate).

### Database layer (`src/database/chats/<operation>/`)

Just `view.rs` + `mod.rs` (no `query.rs` — calls go straight through `SmartDatabase`).
`view.rs` defines a `XxxQueryView` struct holding a `params: Vec<QueryParam>` field and
implementing `mairie360_api_lib::database::db_interface::ApiRequestDto`:
`query_sql(&self) -> &'static str` (raw SQL, params `$1`, `$2`, …), `query_params(&self)
-> &[QueryParam]`, and optionally `cache_key` / `cache_ttl` (none do yet). The struct
`#[derive(serde::Serialize, serde::Deserialize)]` (required by `ApiRequestDto`). Getters
read back out of the `params` vec. IDs are `u64` in the app but `i32`/`i64` in the DB —
cast when building each `QueryParam`.

Endpoints (and `sse::event_manager`) call `state.get_smart_db()` then:

- `fetch_scalar::<T, _>(&view)` — single scalar (`RETURNING id`, `EXISTS`, …). **Writes that
  need "row was found" semantics use `... RETURNING id` + `fetch_scalar`**: 0 rows →
  `Err(ApiLibError::Database(DbError::NotFound))`, which handlers map to 404/400.
- `fetch_all::<T, _>(&view)` / `fetch_one` — the SQL must select **one JSON column**
  (`SELECT to_jsonb(t) FROM (…) t`); the lib decodes each row via `serde_json::from_value`,
  so `T` needs `Serialize + Deserialize` (not `sqlx::FromRow`).
- `execute(view)` — fire-and-forget write, returns `Result<(), ApiLibError>` (no row count).

`QueryParam` has no array variant: `add_users_to_chat` passes the id list as a `"1,2,3"`
`Text` param and expands it in SQL with `unnest(string_to_array($2, ','))::integer`.
There is no compile-time query checking, so schema mistakes surface only at runtime / in
the container tests.

### Real-time SSE (`src/sse/`)

- `state.rs`: `AppState { online_agents: DashMap<u64 /*user id*/, mpsc::Sender<...>>,
  internal_bus: broadcast::Sender<ChatEvent> }`.
- `main.rs` creates a `tokio::sync::broadcast` channel and `tokio::spawn`s
  `event_manager::start_internal_event_listener`, handing it a cloned `SmartDatabase`.
- Write endpoints (e.g. `v1/id/messages/post`) do their DB write, then
  `sse_state.internal_bus.send(ChatEvent { chat_id, sender_id, message })`.
- The listener, per event, queries chat members (`get_chat_users`) and pushes
  `data: {...}\n\n` frames to each online member's `mpsc` sender (skipping the sender).
- `GET /api/v1/stream` registers the caller's `mpsc::Sender` in `online_agents` and runs a
  15s keep-alive ping task that also cleans up the entry on disconnect.

### Config (env vars, all "critical" → process panics if unset)

`REDIS_URL`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`, `DB_NAME`, `HOST`, `PORT`,
`JWT_SECRET`, `JWT_TIMEOUT`. `docker-compose.yml` supplies them for the dev stack (app on
`:3003`, Postgres via `ghcr.io/mairie360/database`, Liquibase migrations, a `seeder` running
`init-test.sql`, Redis, and an nginx front). `.env` is gitignored.

### CI/CD & releases

`.github/workflows/cicd.yml` just calls the reusable `mairie360/CICD` workflow (runs
fmt/clippy/tests, a Postman collection, builds & publishes the image as `message-api`).
Releases use **semantic-release with Angular commit conventions** (`.releaserc.json` /
`release.config.js`): `feat:` → minor, `fix:`/`chore:`/`perf:` → patch, breaking → major.
