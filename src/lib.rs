pub mod database;
pub mod endpoints;
pub mod sse;

// pub fn add_event(chat_id: u64, sender_id: u64, message: &str) {
//     sse::state::AppState::get().update(|state, _| {
//         state.add_event(chat_id, sender_id, message);
//     });
// }
