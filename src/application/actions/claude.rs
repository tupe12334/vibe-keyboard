use std::process::Command;
use tracing::error;

/// `AppleScript` that opens a Terminal.app window and runs `claude` with permissions skipped.
pub(crate) const OPEN_CLAUDE_SCRIPT: &str =
    "tell application \"Terminal\" to do script \"claude --allow-dangerously-skip-permissions \"";

#[allow(
    clippy::zombie_processes,
    reason = "the GUI launcher runs detached and is never waited on"
)]
pub fn open_claude_terminal() {
    Command::new("osascript")
        .arg("-e")
        .arg(OPEN_CLAUDE_SCRIPT)
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Claude in Terminal: {e}");
            std::process::exit(1)
        });
}
