//! Endpoint tests: the real `/api` scope (JwtMiddleware + every route) over the shared test
//! database, called with JWTs signed by `harness::token`. They cover what the query tests cannot:
//! who may call what (MAIR-419).

#[macro_use]
mod harness;
mod access;
mod access_matrix;
mod changes;
mod chat_list;
mod docs;
mod flows;
mod lists;
mod mair_680_session_keys;
mod ready;
mod telemetry;
mod token_refusals;
