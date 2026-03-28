use std::process::Command;
use tracing::error;

pub fn open_claude_terminal() {
    Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Terminal\" to do script \"claude --allow-dangerously-skip-permissions \"")
        .spawn()
        .unwrap_or_else(|e| { error!("Failed to open Claude in Terminal: {e}"); std::process::exit(1) });
}
