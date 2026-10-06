# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`message_api` is the chat/messaging microservice of the **mairie360** platform: an Actix-web
(Rust) HTTP API backed by PostgreSQL and Redis, with a Server-Sent-Events channel for
real-time message notifications. It was bootstrapped from a "Rust API Template" (see
`README.md`); leftover `#change api name` / `#change port` markers in `Cargo.toml`,
`docker-compose.yml` and `nginx.conf` are template residue, not TODOs for a fresh rename.

Most cross-cutting behavior (DB + cache access, auth, env access, query trait, test harness)
lives in the external crate **`mairie360_api_lib`** (crates.io, `1.2.2`, pinned in
`Cargo.lock`). Read its source under `~/.cargo/registry/src/*/mairie360_api_lib-1.2.2/`
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
| Coverage (60% line threshold, excludes `main.rs`/`lib.rs`; `endpoints/` counts since MAIR-419) | `cargo cov_test` (or `cargo cov` for a `codecov.json` report) |
| Regenerate the TS client | `npx orval` (reads `openapi.json` → `generated/`) |
| Run locally | needs all env vars set (see below), then `cargo run` |
| Full dev stack + hot reload | `docker compose up --watch` |

### Tests

Integration tests live in `tests/` (there is no meaningful unit-test suite in `src/`).
`tests/endpoints/` calls the real `/api` scope (`JwtMiddleware` + `endpoints::config`) over the shared test database
with JWTs signed by `harness::token` (the harness sets its own `JWT_SECRET`): it holds the access refusals of every
route (`401`, outsider `404`, non-creator / non-author / administrator `403`, ids of another chat). A new route or a
new refusal gets its test there, since `endpoints/` is part of the coverage gate. `token_refusals.rs` sweeps every
operation of `ApiDoc` declaring `jwt` (`401` without a token, with another scheme, garbage, another secret, an
expired token, `alg: none`, a swapped payload or an asymmetric algorithm; `404` for an unknown or archived account),
and `lists.rs` checks the production quota of `rate_limit_from_env()` (`429` past the burst, `Retry-After` of at
least 1 s).
They are plain `#[tokio::test]` + `#[serial]` (`serial_test`) and use `mairie360_api_lib`'s
`get_shared_db()`, which spins up **real Docker containers via testcontainers** —
`ghcr.io/mairie360/database:3.0.0` (pinned in `.cargo/config.toml` through `TEST_DB_VERSION`, which overrides the lib default) plus a Liquibase
migration container, with Postgres published on a random host port (tests must still stay
`#[serial]`). A running Docker daemon and pull access to `ghcr.io/mairie360/*` are required.

`tests/common::get_smart_db(db_url)` builds a `SmartDatabase` over that Postgres. It also
constructs a `Redis` pointing at `localhost:6379`, but no chat `QueryView` declares a
`cache_key`, so Redis is never actually contacted by the queries. Only `tests/sse/relay.rs` needs Redis: it starts
its own containers (`redis` image through testcontainers, one of them with the platform ACL of the `message-api` role).

```bash
cargo test                                 # whole suite
cargo test --test integration_test         # the one integration binary
cargo test test_create_chat_success        # a single test by name
```

The shared container/seed data is initialized once per run (`OnceCell`), so tests share
one DB and must not depend on a pristine schema.

End-to-end tests (what CI runs on `main` after the dev release, needs Docker + GHCR pull access):

```bash
./integration_test.sh    # docker-compose-integration.yml: full stack + newman replaying tests/postman/collection.json
./security_test.sh       # docker-compose-security.yml: full stack + ZAP scan of /api-docs/openapi.json
./performance_test.sh    # docker-compose-performance.yml: full stack + k6 (load-test.js)
```

The service under test in these three stacks is `image: ${IMAGE_REF}` (no `build:` block). CI sets `IMAGE_REF` to the
published `ghcr.io/mairie360/message-api:dev-<sha>` image; when it is empty the scripts build `message-api:local` from
`development.Dockerfile` first. That image is distroless (no shell, no curl), so readiness is a `message-ready` sidecar
polling `/health`, and dependent services wait for it with `service_completed_successfully`.

Since API_lib 2.0.0 refuses a missing, short (< 32 bytes) or well-known `JWT_SECRET` at startup, every `*_test.sh`
sources `test_secrets.sh` (MAIR-428): a random `JWT_SECRET` per run (unless one is exported) and `ADMIN_JWT`, an HS256
admin token (`sub=1`, 2 h) forged from it with openssl. The compose files require both (`${VAR:?}`); newman gets the
secret as `jwt_secret`, k6 the token as `JWT`. No secret or token valid anywhere is committed. The dev stack keeps
`b"secret"` with `JWT_ALLOW_WEAK_SECRET=true` (dev only).

The ZAP scan is authenticated: `security-scan` injects `ADMIN_JWT` on every request, waits for the `seeder`
service (`init-test.sql`: plain `User` accounts 2 and 3, user 1 is the Admin created by liquibase) and fails on any
alert not set to `IGNORE` / `OUTOFSCOPE` in `.zap/rules.tsv` (no `-I`). `-O http://message:3003` is required: the
spec's `servers` are unreachable from the ZAP container. `rules.tsv` is the one shared by every API, except that the XSS rules (`40012`, `40014`, `40016`, `40017`) are
`IGNORE` since `<` / `>` are accepted in free text (MAIR-426: the API serves JSON with `nosniff`, escaping is the
fronts' job), and that the
`100001` (unexpected content type) scope also covers `/api/v1/stream` (`text/event-stream`): ZAP keeps a single
`OUTOFSCOPE` regex per rule id, so both paths live in one alternation.

Both the ZAP and k6 stacks carry the OpenAPI coverage gate (MAIR-194) from mairie360/CICD `tests/`, available as
`cicd-repo/` (checked out by CI, cloned by the scripts at the pinned `cicd_version` otherwise, override with
`CICD_VERSION`; gitignored). ZAP runs with `--hook zap_hooks.py` and fails when an operation of the served spec was
never reached, or when an operation declaring `security(("jwt" = []))` only got 401/403. `load-test.js` is built on
`coverage.js` and covers every operation (MAIR-195), under a high load on a volume seed (MAIR-474): the performance
stack's `seeder` also runs `init-perf.sql` (2 000 agents `400001`-`402000` in 16 group chats each, 1 000 direct
chats, 25 messages per group chat and 10 000 in the hot chat `100000`; agent `400001 + r` is a member of chat
`100000 + r`). GET handlers run in the `reads` scenario (up to 100 VUs) as a random seeded agent (tokens signed in k6
with the run's `JWT_SECRET`, which the compose file passes): their chat list at a random offset, one of their chats
(the hot one at a random `before`, a third of the time) and its members. The other methods run in the `writes`
scenario (10 VUs) as the Admin, each handler creating and deleting its own chat or message so they are
order-independent; a `chats_rush` scenario sends `GET /api/v1/` at a fixed 100 req/s. `GET /api/v1/stream` has its own 1-VU `stream`
scenario: the SSE response never ends, so k6 cuts it after 1 s (error 1050), a timeout marked expected with
`responseCallback: http.expectedStatuses(0, 200)` so it stays out of `http_req_failed`. One `p(95)` threshold per
`op` tag (200 ms reads, 500 ms writes, 1.5 s stream), `checks > 99%`, `dropped_iterations == 0` and
`http_req_failed < 1%`. Keep `init-perf.sql` and the id ranges at the top of `load-test.js` in step. The spec k6 reads is the one served
by the image under test, saved into the `openapi-spec` volume by `message-ready`. **Adding an endpoint = adding its
handler in `load-test.js`** (k6 aborts at init otherwise), nothing to do for ZAP. `init-test.sql` also seeds the
rows of the spec's path examples (chat 5 created by the Admin with message 118, user 42) so ZAP reaches real rows.
Message 118 is the Admin's own: nobody edits someone else's message, so ZAP would only get `403` on `PATCH` otherwise.

Every `/api/v1/{chat_id}/**` handler starts with `endpoints::v1::id::access::require_chat_access` (one query:
chat exists, caller is a non-excluded member, caller created it (`conversations.created_by`), caller `is_admin`) and
gets a `ChatRole`; anyone who is neither a member nor an administrator gets the same `404` as an unknown chat. The
rules (MAIR-394):

- posting a message is for members only (`ChatRole::require_member`): an administrator who is not a member gets `403`;
- editing a message is for its author only, administrators included (`require_message_author(.., false)`);
- deleting a message: its author, or an administrator (moderation). `DeleteMessageQueryView` writes a
  `DELETE_MESSAGE` row with the content in `messaging_moderation_log` in the same statement when the message is
  someone else's; `DELETE /{chat_id}/` logs `DELETE_CONVERSATION` the same way, whoever deletes;
- `DELETE /{chat_id}/` (MAIR-478, `access::chat_access_in` + `ChatDeleteRightQueryView`): a group chat by its
  creator while still a member, an administrator, or anyone `check_access(user, 'conversations', 'delete', chat)`
  grants (global `delete_all`, individual or group ACL), member or not; a direct chat by an administrator or
  `check_access` only. A member who may not gets `403`, anyone else `404`;
- any member may leave a group chat (`DELETE /{chat_id}/users/{own id}/`); adding members or removing someone else
  needs `ChatRole::can_manage_members` (the creator while still a member, or an administrator), `403` otherwise. A
  chat is deleted with its last member (`DeleteEmptyChatQueryView` after each member removal);
- direct chats (MAIR-478, `conversations.kind = 'direct'` + `direct_user_low` / `direct_user_high`, unique per pair
  since Database `releases/v1.9.0`) are opened by `POST /api/v1/direct/` (`OpenDirectChatQueryView`, find-or-create
  in one statement, shows the chat again to the caller, a new chat stays hidden for the contact). Their participants
  never change: adding members answers `409`, removing the other participant `403` (administrators included).
  "Leaving" only hides the chat (`HideDirectChatQueryView`, `is_excluded = TRUE`); `POST /{chat_id}/messages/`
  runs `RevealDirectChatQueryView` before the insert so both participants see it again (and the unread trigger
  counts the message). Hidden by both, it is deleted by `DeleteEmptyChatQueryView`. `POST /api/v1/` always creates a
  `group` chat. `GET /api/v1/` exposes `kind` and `contact_id` (the other participant, also when hidden);
- reading the chat, its members and `POST /read/` are open to members and administrators.

Write routes (MAIR-420) open a transaction (`access::begin`), check access with `require_chat_access_in` /
`require_message_author_in` inside it, write through the same `SmartTransaction`, then `access::commit`. The access
query is `ChatAccessQueryView::locking`: it takes `FOR KEY SHARE` on the chat row and the caller's membership row,
so a concurrent member removal or chat deletion waits for the write instead of slipping between check and write
(TOCTOU). Removing a member and `DeleteEmptyChatQueryView` share that transaction. SSE events are published after
the commit. Read-only routes keep the plain `require_chat_access`.

`POST /api/v1/` creates the chat and its members in one statement (`CreateChatQueryView::with_members`, a CTE): an
unknown member fails the whole statement, nothing is left behind. `users_id` / `members` go through
`validation::check_user_ids` (at most 50, ids 1..=`i32::MAX`, no duplicate).

`GET /api/v1/{chat_id}/` is paginated by keyset on the message id (`?before=<id>&limit=<1..100, default 50>`): the
query fetches `limit + 1` rows newest first, `GetChatResultView::from_newest_first` trims the extra one into
`has_more` / `next_before` and returns the page oldest first. `citation` is stored in `messages.reply_to_id`, whose
composite foreign key `(conversation_id, reply_to_id)` rejects a message of another chat (mapped to `400`).
No `QueryParam` holds an optional BIGINT, so an absent `before` / `citation` travels as `0` (`NULLIF`).

Unread counters are only lowered by the explicit `POST /api/v1/{chat_id}/read/` (`{ "readUntilMessageId": n }`, answers
`{ "unread_count": k }`); `GET /api/v1/{chat_id}/` and `GET /api/v1/` never touch them, so polling is safe. The route is
`endpoints::v1::id::read` and one call to the Postgres function `fn_acknowledge_read` (`database::chats::acknowledge_read`),
which lives in `Devops/Database` (`releases/v1.5.0` + `repeatable/messages/`): it moves a per-agent cursor
(`conversation_read_cursors`) forward only, recounts the messages after it, and answers "no row" (→ `404 Unknown message.`)
when the id belongs to another chat. Sends and acknowledgements of one conversation are serialized by a transaction advisory
lock taken by a `BEFORE INSERT` trigger on `messages`, which also draws the message id after the lock so a single cursor is
sound. **This API needs a Database image that ships that release, `releases/v1.8.0`** (MAIR-394: `created_by`,
`reply_to_id`, `messaging_moderation_log`) **and `releases/v1.9.0`** (MAIR-478: direct chat pair): the compose files and
`TEST_DB_VERSION` in `.cargo/config.toml` pin it, and both have to be bumped together when a newer Database image is
needed. To try an unmerged Database branch, build `ghcr.io/mairie360/database:<tag>` and `…/liquibase-migrations:<tag>`
from `Devops/Database` and run `TEST_DB_VERSION=<tag> cargo test`.

Logging (MAIR-421) goes through `tracing` (`src/logging.rs`, initialized first thing in `main`): JSON lines by
default, `LOG_FORMAT=text` for readable lines (dev stack), level from `RUST_LOG` (default `info`). `TracingLogger`
(`tracing-actix-web`) opens one span per request with its `request_id`, so a handler's log carries it. Database errors
go through `endpoints::error::classify(operation, e)`: `NotFound` / `ForeignKey` / `Unique` come back for the
handler to map to its documented `4xx`, anything else is logged at `error` with the operation and becomes `500`
(`unexpected` logs unconditionally, for statements where no client error is possible). No `eprintln!`, no
`.map_err(|_| …)`.

Request bodies with text fields are extracted with `endpoints::validation::ValidatedJson` instead of `web::Json`:
the view implements `Validate` (length matching the Postgres column, no control character) and an
invalid value answers `400` naming the field. Map the lib's `DbError` constraint violations (`ForeignKeyViolation`,
`UniqueViolation`) to `4xx` instead of `500`.

`tests/postman/collection.json` is a Postman v2.1 collection (importable in the app) and
`tests/postman/environment.json` its variables; the compose file overrides `baseUrl` with `--env-var` so the
committed default (`http://localhost:3003`) stays usable from a host shell. There is no login route here, so the
collection pre-request script forges the HS256 JWTs itself (claims `sub`/`role`/`exp`, signed with the stack's
`JWT_SECRET`) for the seeded Admin (user 1) and a plain user (user 2, from `init-test.sql`). The scenario creates
its own chat and deletes it at the end, so it is replayable against a persistent database. `GET /api/v1/stream`
is only checked unauthenticated (401): newman cannot consume an open SSE stream.

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

`src/main.rs` mounts `/health` (liveness) + `/ready` (readiness) publicly, Swagger UI + `/api-docs/openapi.json`
only when `SWAGGER_ENABLED=true` (`swagger::docs_config`, MAIR-424: set by the dev, ZAP and k6 stacks, never on a
deployed instance nor on the integration stack, whose Postman collection checks the `404`), and everything else under
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
convert with `database::ids` (`id_to_sql` / `id_from_sql` for `INTEGER`, `bigint_to_sql` / `bigint_from_sql` for
`messages.id`), never with `as`: an `as i32` wraps `chat + 2^32` around to `chat` (MAIR-422). The saturating
conversions turn an out-of-range id into one no row has, so it ends in the usual `404`. `src/lib.rs` enables the
clippy cast lints, so `cargo check_code` rejects a new `as` cast.

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

- `state.rs`: `AppState { online_agents: DashMap<u64 /*user id*/, Vec<Connection>>, internal_bus:
  broadcast::Sender<ChatEvent>, relay: Option<Arc<RedisRelay>> }`, built with `AppState::new`. A `Connection` holds its
  sender and a `close: Arc<Notify>`. `register` keeps at most `MAX_CONNECTIONS_PER_USER` (5) per user and notifies the
  evicted oldest ones.
- `relay.rs`: Redis pub/sub so every replica notifies its own streams. The channel is
  `<lib key prefix>:sse:chat-events` (`message-api:sse:chat-events` on the platform, whose Redis ACL grants that role
  `&message-api:*` + `PUBLISH`/`SUBSCRIBE`, `redis.pubSubRoles` in Deploiment). `RedisRelay::run` subscribes and pushes
  what it receives on `internal_bus`, reconnecting with a backoff; redis-rs drops the error reply of `SUBSCRIBE`, so the
  subscription is only trusted once a `probe` payload published right after it comes back.
- `main.rs` creates the `broadcast` channel, spawns `RedisRelay::run` and
  `event_manager::start_internal_event_listener` (with a cloned `SmartDatabase`).
- Write endpoints do their DB write, then `sse_state.publish(ChatEvent { chat_id, sender_id })`: through Redis when the
  relay is subscribed, straight on `internal_bus` otherwise (a duplicate signal only makes a client reload once more).
- The listener, per event, queries the non-excluded chat members (`get_chat_users`) and pushes a serde-serialized
  `ChatSignal` frame (`data: {...}\n\n`) to every connection of each online member (skipping the
  sender). A `Lagged` bus error is logged and the loop resumes; only closed senders are removed.
- `GET /api/v1/stream` registers a new connection for the caller and spawns a supervisor (`actix_web::rt::spawn`): 15 s
  keep-alive ping, `authenticate_token` again every 30 s (revoked session, deleted account), and a timer on the JWT
  `exp` when it is readable. Any failure notifies `close`, which ends the response (`take_until`); the connection is
  removed once its receiver is gone.

### Lists and rate limiting (MAIR-425)

`GET /api/v1/` and `GET /api/v1/{chat_id}/users/` take `endpoints::pagination::PageQuery` (`limit` 1..=100, default 50,
`offset`) and answer `has_more`: the query views' `page()` fetch `limit + 1` rows and `split_page` trims the extra one.
Their `new()` stays unbounded (`LIMIT NULL`) for the SSE fan-out and the query tests. Messages use their own keyset.

`/api` is rate limited per authenticated user (`endpoints::rate_limit`, `actix-governor`): the BFFs share a few IPs,
so the key is the `AuthenticatedUser` that `JwtMiddleware` stored, hence `rate_limiter()` is wrapped *before*
`JwtMiddleware` (inner). `RATE_LIMIT_PER_SECOND` (default 10, `0` disables) and `RATE_LIMIT_BURST` (default 50); `CallerKey` answers the
`429` itself so that `Retry-After` / `X-RateLimit-After` say at least 1 s (actix-governor rounds them down to `0`);
the ZAP and k6 stacks disable it. `swagger::RateLimitAddon` adds the `429` to every operation with
`security(("jwt" = []))`, so handlers do not declare it.

### Probes and startup (MAIR-423)

`GET /health` is the liveness probe (always `OK`, touches nothing). `GET /ready` (`endpoints/ready.rs`) is the
readiness probe: Postgres `SELECT 1` and Redis `EXISTS message-api:readiness` (the platform ACL role has no `PING`),
2 s each, `200 {"postgres":"up","redis":"up"}` or `503` naming the dependency down. The test stacks' `message-ready`
sidecars and the dev healthcheck wait on `/ready`; the chart probes are in Deploiment. API_lib starts without a
database, so `main` calls `ready::wait_for_postgres` and exits when Postgres does not answer within
`DB_STARTUP_TIMEOUT` seconds (default 60). Neither probe is registered under `/api`.

### Config (env vars, all "critical" → process panics if unset)

`REDIS_URL`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`, `DB_NAME`, `HOST`, `PORT`,
`JWT_SECRET`, `JWT_TIMEOUT`. Optional: `SWAGGER_ENABLED`, `DB_STARTUP_TIMEOUT`, `RATE_LIMIT_PER_SECOND`, `RATE_LIMIT_BURST`, `LOG_FORMAT`, `RUST_LOG`. The Postgres URL is assembled by `database::pg_url::build_pg_url`,
which percent-encodes user, password and database name, so `DB_PASSWORD` may contain any
character. `docker-compose.yml` supplies them for the dev stack (app on
`:3003`, Postgres via `ghcr.io/mairie360/database`, Liquibase migrations, a `seeder` running
`init-test.sql`, Redis, and an nginx front). `.env` is gitignored.

### CI/CD & releases

`.github/workflows/cicd.yml` just calls the reusable `mairie360/CICD` workflow (runs
fmt/clippy/tests, the three `*_test.sh` stacks run against the published `dev-<sha>` image through `IMAGE_REF`, builds & publishes the image as `message-api`).
Releases use **semantic-release with Angular commit conventions** (`.releaserc.json` /
`release.config.js`): `feat:` → minor, `fix:`/`chore:`/`perf:` → patch, breaking → major.

Kept in sync with `API_template` (MAIR-427): `cicd.yml` passes only the secrets the reusable workflow declares
(`CODECOV_TOKEN`, `N8N_WEBHOOK_SECRET`, no `secrets: inherit`) and Renovate bumps the `uses:` tag and `cicd_version`
in one grouped PR. Both Dockerfiles pin their base images by digest on the template's Rust version; the production
one builds the dependencies in a cached layer, then the crate, both with `--locked`. The dev image runs as `dev`
(uid 1000). Every advisory ignored in `.cargo/audit.toml` carries why it does not apply and when to drop it.

## Pull request reviewers

Every PR requests a review from the whole team, minus its author: `CarolinHugo`, `LAURETbenjamin`, `MathTek` and `Quentintnrl` (`gh pr create … --reviewer CarolinHugo,LAURETbenjamin,MathTek`). `.github/CODEOWNERS` makes GitHub request them automatically as well.
