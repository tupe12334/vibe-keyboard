#![allow(clippy::expect_used)]
use super::*;
use crate::domain::profile::Profile;
use std::sync::PoisonError;

fn temp_path(name: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("vibe-keyboard-setup-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("profile.toml");
    let _ = std::fs::remove_file(&path);
    path
}

fn empty_state() -> Arc<Mutex<AppState>> {
    Arc::new(Mutex::new(AppState::new(0)))
}

#[test]
fn load_reads_the_saved_pages_into_the_state() {
    let path = temp_path("saved");
    let mut profile = Profile::default();
    profile.add_page("First");
    profile.add_page("Second");
    crate::infrastructure::profile_store::save(&path, &profile).expect("save profile");

    let state = empty_state();
    load_profile_at(&state, &path);

    let s = state.lock().unwrap_or_else(PoisonError::into_inner);
    assert_eq!(s.total_pages, 2);
    assert_eq!(s.profile.pages.len(), 2);
    assert_eq!(s.profile.pages[1].name, "Second");
}

#[test]
fn load_writes_the_default_profile_only_when_the_file_is_missing() {
    let path = temp_path("missing");
    let state = empty_state();
    load_profile_at(&state, &path);
    assert!(path.exists(), "default profile written");
    assert!(
        state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .total_pages
            >= 1
    );
}
