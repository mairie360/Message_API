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
mod mair_1106_unread_total;
mod mair_1155_messaging_v2;
mod mair_1219_deleted_quotes;
mod mair_534_before;
mod mair_657_websocket;
mod mair_680_session_keys;
mod path_ids;
mod ready;
mod telemetry;
mod token_refusals;
