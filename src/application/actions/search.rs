use std::process::{Command, Stdio};
use tracing::error;

/// Open macOS Spotlight search.
#[allow(clippy::zombie_processes, reason = "fire-and-forget OS application launcher: the spawned process is the target app itself, which is expected to outlive this binary")]
pub fn open_spotlight() {
    Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to key code 49 using {command down}")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Spotlight: {e}");
            std::process::exit(1)
        });
}
