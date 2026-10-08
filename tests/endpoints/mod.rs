//! Endpoint tests: the real `/api` scope (JwtMiddleware + every route) over the shared test
//! database, called with JWTs signed by `harness::token`. They cover what the query tests cannot:
//! who may call what (MAIR-419).

#[macro_use]
mod harness;
mod access;
mod docs;
mod flows;
mod lists;
mod ready;
mod telemetry;
mod token_refusals;
