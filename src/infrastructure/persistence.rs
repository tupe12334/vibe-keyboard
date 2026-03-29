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
        if let Ok(contents) = toml::to_string_pretty(self) {
            if let Err(e) = fs::write(&path, contents) {
                error!("failed to write {}: {e}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_temp_config<F: FnOnce()>(f: F) {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!(
            "vibe-kb-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        std::fs::remove_dir_all(&dir).ok();
        // SAFETY: guarded by ENV_LOCK, no signal handlers in tests
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &dir) };
        f();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn default_values() {
        let s = DeviceState::default();
        assert_eq!(s.current_page, 0);
        assert_eq!(s.brightness, 25);
    }

    #[test]
    fn load_creates_defaults_when_missing() {
        with_temp_config(|| {
            let s = DeviceState::load();
            assert_eq!(s.current_page, 0);
            assert_eq!(s.brightness, 25);
        });
    }

    #[test]
    fn save_and_load_roundtrip() {
        with_temp_config(|| {
            let s = DeviceState {
                current_page: 1,
                brightness: 50,
            };
            s.save();
            let loaded = DeviceState::load();
            assert_eq!(loaded.current_page, 1);
            assert_eq!(loaded.brightness, 50);
        });
    }

    #[test]
    fn load_returns_defaults_on_invalid_toml() {
        with_temp_config(|| {
            let path = state_path();
            let parent = path.parent().expect("path has parent");
            std::fs::create_dir_all(parent).unwrap_or_else(|_| ());
            std::fs::write(&path, b"not valid toml ][").unwrap_or_else(|_| ());
            let s = DeviceState::load();
            assert_eq!(s.current_page, 0);
            assert_eq!(s.brightness, 25);
        });
    }

    #[test]
    fn load_uses_home_when_xdg_not_set() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let home_dir = std::env::temp_dir().join(format!(
            "vibe-kb-home-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        std::fs::remove_dir_all(&home_dir).ok();
        // SAFETY: guarded by ENV_LOCK, no signal handlers in tests
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
        unsafe { std::env::set_var("HOME", &home_dir) };
        let s = DeviceState::load();
        assert_eq!(s.current_page, 0);
        assert_eq!(s.brightness, 25);
        unsafe { std::env::remove_var("HOME") };
        std::fs::remove_dir_all(&home_dir).ok();
    }

    #[test]
    fn save_failure_on_blocked_dir() {
        with_temp_config(|| {
            // Block the vibe-keyboard directory by placing a file there
            let path = state_path();
            let config_root = path
                .parent()
                .and_then(|p| p.parent())
                .expect("path has grandparent");
            std::fs::create_dir_all(config_root).unwrap_or_else(|_| ());
            let vibe_dir = path.parent().expect("path has parent");
            // Create a file at vibe_dir's location — create_dir_all will fail
            std::fs::write(vibe_dir, b"block").unwrap_or_else(|_| ());

            let s = DeviceState {
                current_page: 0,
                brightness: 25,
            };
            s.save(); // gracefully logs error, does not panic
        });
    }

    #[test]
    fn save_failure_on_readonly_file() {
        with_temp_config(|| {
            // First create the file via a successful save
            DeviceState::default().save();

            // Make it read-only so the next write fails
            let path = state_path();
            if let Ok(meta) = std::fs::metadata(&path) {
                let mut perms = meta.permissions();
                perms.set_readonly(true);
                std::fs::set_permissions(&path, perms).unwrap_or_else(|_| ());
            }

            DeviceState {
                current_page: 1,
                brightness: 75,
            }
            .save(); // logs error, no panic

            // Restore so cleanup can proceed
            if let Ok(meta) = std::fs::metadata(&path) {
                let mut perms = meta.permissions();
                perms.set_readonly(false);
                std::fs::set_permissions(&path, perms).unwrap_or_else(|_| ());
            }
        });
    }
}
