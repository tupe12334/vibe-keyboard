use super::*;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(super) fn env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

pub(super) fn with_temp_config<F: FnOnce()>(f: F) {
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
