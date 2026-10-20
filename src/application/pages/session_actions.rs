/// Text that key 14 sends to a session. Fixed, so it stays off the prompt list.
pub const RECAP_TEXT: &str = "/recap";

#[path = "session_actions_keys.rs"]
mod keys;
#[path = "session_actions_render.rs"]
mod render;

pub use keys::open_session_actions;
pub use render::render_session_actions;

#[cfg(test)]
#[path = "session_actions_tests.rs"]
mod tests;
