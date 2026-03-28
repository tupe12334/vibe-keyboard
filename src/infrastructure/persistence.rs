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

fn config_dir() -> PathBuf {
    // ~/.config on Linux/macOS
    if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config);
    }
    let mut home = PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| "/tmp".into()));
    home.push(".config");
    home
}

impl DeviceState {
    /// Load state from `~/.config/vibe-keyboard/state.toml`.
    /// Returns defaults if the file is missing or unreadable.
    pub fn load() -> Self {
        let path = state_path();
        match fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
                warn!("failed to parse {}: {e}, using defaults", path.display());
                Self::default()
            }),
            Err(_) => {
                let state = Self::default();
                state.save(); // write defaults on first run
                state
            }
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
        match toml::to_string_pretty(self) {
            Ok(contents) => {
                if let Err(e) = fs::write(&path, contents) {
                    error!("failed to write {}: {e}", path.display());
                }
            }
            Err(e) => error!("failed to serialize state: {e}"),
        }
    }
}
