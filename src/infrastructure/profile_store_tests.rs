use super::{profile_guide::GUIDE_README, *};
use crate::domain::profile::{Action, KeyConfig};
use std::os::unix::fs::PermissionsExt;

/// Root ignores file modes, so permission-denied cases cannot fail as expected.
pub(super) fn running_as_root() -> bool {
    std::env::var("USER").is_ok_and(|user| user == "root")
}

pub(super) struct TempDir(pub(super) PathBuf);

impl TempDir {
    pub(super) fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let dir =
            std::env::temp_dir().join(format!("vibe-kb-{tag}-{}-{nanos}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(super) fn read_ok(path: &Path) -> Profile {
    read(path).ok().flatten().unwrap_or_default()
}

#[test]
fn paths_point_into_the_vibe_keyboard_config_dir() {
    assert!(profile_path().ends_with("vibe-keyboard/profile.toml"));
    assert!(root_dir().ends_with("vibe-keyboard"));
}

#[test]
fn startup_keeps_an_existing_profile() {
    let dir = TempDir::new("exists");
    let path = dir.0.join("profile.toml");
    let mut edited = Profile::default_from_legacy();
    edited.add_page("Extra");
    save(&path, &edited).unwrap_or_default();
    assert_eq!(ensure_default(&path).ok(), Some(edited));
}

#[test]
fn startup_writes_legacy_layout_when_missing() {
    let dir = TempDir::new("first-run");
    let path = dir.0.join("profile.toml");
    assert_eq!(
        ensure_default(&path).ok(),
        Some(Profile::default_from_legacy())
    );
    assert!(path.exists());
}

#[test]
fn reload_read_never_creates_the_file() {
    let dir = TempDir::new("no-create");
    let path = dir.0.join("profile.toml");
    assert_eq!(read(&path), Ok(None));
    assert!(!path.exists());
}

#[cfg(test)]
#[path = "profile_guide_tests.rs"]
mod guide_tests;

#[cfg(test)]
#[path = "profile_store_save_tests.rs"]
mod save_tests;

#[cfg(test)]
#[path = "profile_store_images_tests.rs"]
mod images_tests;
