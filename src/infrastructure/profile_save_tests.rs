#![allow(
    clippy::expect_used,
    reason = "test module: setup failures should panic"
)]

use super::*;
use std::fs::File;
use std::time::SystemTime;

fn temp_profile(name: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("vibe-keyboard-save-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("profile.toml");
    let _ = std::fs::remove_file(&path);
    std::fs::write(&path, "").expect("seed file");
    path
}

/// Another editor (or the user) edits the file. The modification time is put back.
fn external_edit_keeping_time(path: &Path) {
    let before = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    std::fs::write(path, "[external]\n").expect("external edit");
    File::options()
        .write(true)
        .open(path)
        .and_then(|f| f.set_modified(before))
        .expect("restore mtime");
}

#[test]
fn refusals_read_as_messages() {
    assert!(SaveRefusal::Changed.to_string().contains("changed on disk"));
    assert_eq!(SaveRefusal::Failed("boom".into()).to_string(), "boom");
}

#[test]
fn a_lock_file_that_cannot_open_refuses_without_writing() {
    let path = std::env::temp_dir()
        .join("vibe-keyboard-no-such-dir")
        .join("profile.toml");
    let profile = Profile::default_from_legacy();
    assert!(matches!(
        save_if_unchanged(&path, &profile, None),
        Err(SaveRefusal::Failed(_))
    ));
    assert!(!path.exists());
}

#[test]
fn writes_when_the_file_is_as_loaded_and_returns_its_new_version() {
    let path = temp_profile("guard");
    let (profile, loaded) = load_with_version(&path).expect("load");
    let written = save_if_unchanged(&path, &profile, loaded).expect("save");
    assert_eq!(written, version_of(&path));
    assert!(written.is_some());
}

#[test]
fn a_concurrent_external_edit_after_load_is_refused() {
    let path = temp_profile("external");
    let (profile, loaded) = load_with_version(&path).expect("load");
    external_edit_keeping_time(&path);
    assert_eq!(
        save_if_unchanged(&path, &profile, loaded),
        Err(SaveRefusal::Changed)
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "[external]\n"
    );
}

#[test]
fn a_stale_version_is_refused() {
    let path = temp_profile("stale");
    let profile = Profile::default_from_legacy();
    let stale = version_of(&path).map(|v| v.wrapping_add(1));
    assert_eq!(
        save_if_unchanged(&path, &profile, stale),
        Err(SaveRefusal::Changed)
    );
}

#[path = "profile_save_version_tests.rs"]
mod version_tests;
