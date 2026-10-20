use super::*;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

fn with_temp_config<F: FnOnce()>(f: F) {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = std::env::temp_dir().join(format!(
        "vibe-kb-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos())
    ));
    let _ = std::fs::remove_dir_all(&dir);
    // SAFETY: guarded by ENV_LOCK, no signal handlers in tests
    unsafe { std::env::set_var("XDG_CONFIG_HOME", &dir) };
    f();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    let _ = std::fs::remove_dir_all(&dir);
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
        let _ = std::fs::create_dir_all(parent);
        let _ = std::fs::write(&path, b"not valid toml ][");
        let s = DeviceState::load();
        assert_eq!(s.current_page, 0);
        assert_eq!(s.brightness, 25);
    });
}

#[test]
fn config_dir_is_dot_config_under_home() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let old_home = std::env::var_os("HOME");
    // SAFETY: guarded by ENV_LOCK, no signal handlers in tests
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", "/home/tester") };
    assert_eq!(config_dir(), PathBuf::from("/home/tester/.config"));
    unsafe { std::env::set_var("XDG_CONFIG_HOME", "/xdg") };
    assert_eq!(config_dir(), PathBuf::from("/xdg"));
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    match old_home {
        Some(home) => unsafe { std::env::set_var("HOME", home) },
        None => unsafe { std::env::remove_var("HOME") },
    }
}

#[test]
fn load_uses_home_when_xdg_not_set() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let home_dir = std::env::temp_dir().join(format!(
        "vibe-kb-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos())
    ));
    let _ = std::fs::remove_dir_all(&home_dir);
    // SAFETY: guarded by ENV_LOCK, no signal handlers in tests
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", &home_dir) };
    let s = DeviceState::load();
    assert_eq!(s.current_page, 0);
    assert_eq!(s.brightness, 25);
    unsafe { std::env::remove_var("HOME") };
    let _ = std::fs::remove_dir_all(&home_dir);
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
        let _ = std::fs::create_dir_all(config_root);
        let vibe_dir = path.parent().expect("path has parent");
        // Create a file at vibe_dir's location — create_dir_all will fail
        let _ = std::fs::write(vibe_dir, b"block");

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
            let _ = std::fs::set_permissions(&path, perms);
        }

        DeviceState {
            current_page: 1,
            brightness: 75,
        }
        .save(); // logs error, no panic

        // Restore so cleanup can proceed
        if let Ok(meta) = std::fs::metadata(&path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o644);
            let _ = std::fs::set_permissions(&path, perms);
        }
    });
}
