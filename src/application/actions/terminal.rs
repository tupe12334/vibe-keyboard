use std::process::Command;
use tracing::error;

pub fn open_terminal() {
    Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Terminal\" to do script \"\"")
        .spawn()
        .unwrap_or_else(|e| { error!("Failed to open Terminal: {e}"); std::process::exit(1) });
}
