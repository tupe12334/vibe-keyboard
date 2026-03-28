use std::process::Command;
use tracing::error;

#[allow(clippy::zombie_processes)]
pub fn open_config_in_vscode() {
    let config_path = {
        let mut p = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            std::path::PathBuf::from(xdg)
        } else {
            let mut home =
                std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into()));
            home.push(".config");
            home
        };
        p.push("vibe-keyboard");
        p.push("state.toml");
        p
    };
    Command::new("code")
        .arg(config_path)
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open config in VS Code: {e}");
            std::process::exit(1)
        });
}

#[allow(clippy::zombie_processes)]
pub fn open_vscode_in_path(path: &str) {
    Command::new("code").arg(path).spawn().unwrap_or_else(|e| {
        error!("Failed to open VS Code at {path}: {e}");
        std::process::exit(1)
    });
}
