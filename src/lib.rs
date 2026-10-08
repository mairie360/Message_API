// Ids are `u64` in the API and INTEGER / BIGINT in Postgres: a plain `as` cast wraps around
// (`4294967301 as i32 == 5`), which made an out-of-range id alias another row (MAIR-422). Convert
// with `database::ids` instead.
#![warn(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

pub mod database;
pub mod endpoints;
pub mod logging;
pub mod request_log;
pub mod sse;

// pub fn add_event(chat_id: u64, sender_id: u64, message: &str) {
//     sse::state::AppState::get().update(|state, _| {
//         state.add_event(chat_id, sender_id, message);
//     });
// }
