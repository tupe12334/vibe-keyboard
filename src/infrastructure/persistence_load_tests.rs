use super::tests::{env_lock, with_temp_config};
use super::*;

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
