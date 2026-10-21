use std::process::{Command, Stdio};
use tracing::error;

/// `AppleScript` that opens a Terminal.app window with an empty shell.
pub(crate) const OPEN_TERMINAL_SCRIPT: &str = "tell application \"Terminal\" to do script \"\"";

/// `AppleScript` that opens a Terminal.app window and `cd`s into `path`, or stays empty without one.
pub(crate) fn terminal_in_path_script(path: Option<&str>) -> String {
    match path {
        Some(p) => format!("tell application \"Terminal\" to do script \"cd '{p}'\""),
        None => OPEN_TERMINAL_SCRIPT.to_string(),
    }
}

#[allow(
    clippy::zombie_processes,
    reason = "the GUI launcher runs detached and is never waited on"
)]
pub fn open_terminal() {
    Command::new("osascript")
        .arg("-e")
        .arg(OPEN_TERMINAL_SCRIPT)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Terminal: {e}");
            std::process::exit(1)
        });
}

#[allow(
    clippy::zombie_processes,
    reason = "the GUI launcher runs detached and is never waited on"
)]
pub fn open_terminal_in_path(path: Option<&str>) {
    let script = terminal_in_path_script(path);
    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Terminal: {e}");
            std::process::exit(1)
        });
}
