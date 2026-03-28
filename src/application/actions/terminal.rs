use std::process::{Command, Stdio};
use tracing::error;

#[allow(clippy::zombie_processes)]
pub fn open_terminal() {
    Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Terminal\" to do script \"\"")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Terminal: {e}");
            std::process::exit(1)
        });
}

#[allow(clippy::zombie_processes)]
pub fn open_terminal_in_path(path: Option<&str>) {
    let script = match path {
        Some(p) => format!("tell application \"Terminal\" to do script \"cd '{p}'\""),
        None => "tell application \"Terminal\" to do script \"\"".to_string(),
    };
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
