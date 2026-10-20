use tracing::info;

use super::{Plugin, PluginHost};
use crate::domain::navigation::Screen;

/// Opens the Claude Code session list. No tile: its page draws the session keys.
pub struct ClaudeSessions;

impl Plugin for ClaudeSessions {
    fn name(&self) -> &'static str {
        "claude_sessions"
    }
    fn label(&self) -> &'static str {
        "Claude sessions"
    }
    fn run(&self, host: &mut dyn PluginHost) {
        host.set_loading(true);
        let sessions = host.list_sessions();
        host.set_loading(false);
        info!("claude sessions: {} found", sessions.len());
        host.show(Screen::ClaudeSessions { sessions, page: 0 });
    }
}
