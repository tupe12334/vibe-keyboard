use std::process::Command;
use tracing::error;

pub fn open_terminal() {
    Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Terminal\" to do script \"\"")
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Terminal: {e}");
            std::process::exit(1)
        });
}

pub fn open_terminal_in_path(path: Option<&str>) {
    let script = match path {
        Some(p) => format!("tell application \"Terminal\" to do script \"cd '{p}'\""),
        None => "tell application \"Terminal\" to do script \"\"".to_string(),
    };
    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Terminal: {e}");
            std::process::exit(1)
        });
}
