use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    pub current_page: usize,
    pub brightness: u8,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            current_page: 0,
            brightness: 25,
        }
    }
}

fn state_path() -> PathBuf {
    let mut p = config_dir();
    p.push("vibe-keyboard");
    p.push("state.toml");
    p
}

/// `$XDG_CONFIG_HOME`, or `$HOME/.config` on every OS. `dirs::config_dir()` is not used,
/// because on macOS it returns `~/Library/Application Support`.
pub fn config_dir() -> PathBuf {
    if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .unwrap_or_else(std::env::temp_dir);
    home.join(".config")
}

impl DeviceState {
    /// Load state from `~/.config/vibe-keyboard/state.toml`.
    /// Returns defaults if the file is missing or unreadable.
    pub fn load() -> Self {
        let path = state_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_else(|e| {
                warn!("failed to parse {}: {e}, using defaults", path.display());
                Self::default()
            })
        } else {
            let state = Self::default();
            state.save(); // write defaults on first run
            state
        }
    }

    /// Persist state to `~/.config/vibe-keyboard/state.toml`.
    pub fn save(&self) {
        let path = state_path();
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("failed to create config dir: {e}");
                return;
            }
        }
        if let Ok(contents) = toml::to_string_pretty(self) {
            if let Err(e) = fs::write(&path, contents) {
                error!("failed to write {}: {e}", path.display());
            }
        }
    }
}

#[cfg(test)]
#[path = "persistence_tests.rs"]
#[allow(
    clippy::expect_used,
    reason = "test module: setup failures should panic"
)]
mod tests;

#[cfg(test)]
#[path = "persistence_load_tests.rs"]
#[allow(
    clippy::expect_used,
    reason = "test module: setup failures should panic"
)]
mod load_tests;
