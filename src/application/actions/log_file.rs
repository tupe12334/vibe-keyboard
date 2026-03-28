use std::process::Command;
use tracing::error;

#[allow(clippy::zombie_processes)]
pub fn open_log_file() {
    let log_path = {
        let mut p = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            std::path::PathBuf::from(xdg)
        } else {
            let mut home =
                std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into()));
            home.push(".config");
            home
        };
        p.push("vibe-keyboard");
        p.push("app.log");
        p
    };
    Command::new("code")
        .arg(log_path)
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open log file in VS Code: {e}");
            std::process::exit(1)
        });
}
