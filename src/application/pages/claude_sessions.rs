use crate::domain::actions::SessionStatus;

mod nav_keys;
mod refresh;
mod render;
pub use refresh::SessionsRefresh;
pub use render::render_claude_sessions;

/// Short status for the button description, shown in the editor and the TUI.
const fn status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "Active",
        SessionStatus::Stale => "Stale",
    }
}
