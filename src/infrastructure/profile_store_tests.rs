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

#[test]
fn load_save_is_byte_identical_and_stable() {
    let dir = TempDir::new("bytes");
    let path = dir.0.join("profile.toml");
    save(&path, &Profile::default_from_legacy()).unwrap_or_default();
    let first = fs::read_to_string(&path).unwrap_or_default();

    let loaded = read_ok(&path);
    save(&path, &loaded).unwrap_or_default();
    let second = fs::read_to_string(&path).unwrap_or_default();
    save(&path, &loaded).unwrap_or_default();
    let third = fs::read_to_string(&path).unwrap_or_default();

    assert_eq!(first, second);
    assert_eq!(second, third);
    assert!(first.ends_with('\n'));
}

#[test]
fn malformed_file_reports_line_and_column() {
    let dir = TempDir::new("bad");
    let path = dir.0.join("profile.toml");
    fs::write(&path, "[[pages]\nname = \"x\"\n").unwrap_or_default();
    let err = read(&path).err().unwrap_or_default();
    assert!(err.contains("line"), "error was: {err}");
}

#[test]
fn relative_image_resolves_against_root() {
    let root = Path::new("/cfg/vibe-keyboard");
    assert_eq!(
        resolve_image(root, Path::new("images/a.png")),
        PathBuf::from("/cfg/vibe-keyboard/images/a.png")
    );
    assert_eq!(
        resolve_image(root, Path::new("/abs/b.png")),
        PathBuf::from("/abs/b.png")
    );
}

#[test]
fn import_copies_outside_files_and_keeps_inside_paths() {
    let root = TempDir::new("root");
    let outside = TempDir::new("outside");
    let src = outside.0.join("pic.png");
    fs::write(&src, b"one").unwrap_or_default();

    let rel = import_image(&root.0, &src).unwrap_or_default();
    assert_eq!(rel, PathBuf::from("images/pic.png"));
    assert_eq!(fs::read(root.0.join(&rel)).unwrap_or_default(), b"one");

    // Same bytes reuse the copy; different bytes get a numbered name.
    assert_eq!(import_image(&root.0, &src).unwrap_or_default(), rel);
    fs::write(&src, b"two").unwrap_or_default();
    let second = import_image(&root.0, &src).unwrap_or_default();
    assert_eq!(second, PathBuf::from("images/pic-1.png"));

    let inside = root.0.join("images/pic.png");
    assert_eq!(import_image(&root.0, &inside).unwrap_or_default(), rel);
}

#[test]
fn import_resolves_parent_components_before_checking_the_root() {
    let root = TempDir::new("dotdot");
    let images = root.0.join("images");
    fs::create_dir_all(&images).unwrap_or_default();
    fs::write(images.join("a.png"), b"a").unwrap_or_default();
    let sneaky = root
        .0
        .join("images")
        .join("..")
        .join("images")
        .join("a.png");
    assert_eq!(
        import_image(&root.0, &sneaky).unwrap_or_default(),
        PathBuf::from("images/a.png")
    );
}

#[test]
fn saved_profile_keeps_relative_image_paths() {
    let dir = TempDir::new("rel");
    let path = dir.0.join("profile.toml");
    let mut profile = Profile::default_from_legacy();
    profile.set_key(
        0,
        5,
        KeyConfig {
            image: Some(PathBuf::from("images/claude.png")),
            action: Action::None,
            ..KeyConfig::default()
        },
    );
    save(&path, &profile).unwrap_or_default();
    let text = fs::read_to_string(&path).unwrap_or_default();
    assert!(text.contains("image = \"images/claude.png\""));
    assert!(!text.contains("/Users/"));
}

#[test]
fn io_failures_are_reported_not_panicked() {
    let dir = TempDir::new("io");
    let profile = Profile::default_from_legacy();
    // A directory where the file should be: read fails, rename fails, temp write fails.
    let as_dir = dir.0.join("as-dir.toml");
    fs::create_dir_all(&as_dir).unwrap_or_default();
    assert!(read(&as_dir).is_err());
    assert!(save(&as_dir, &profile).is_err());
    let blocked = dir.0.join("blocked.toml");
    fs::create_dir_all(blocked.with_extension("toml.tmp")).unwrap_or_default();
    assert!(save(&blocked, &profile).is_err());
    // A file where the parent directory should be.
    assert!(ensure_default(&as_dir).is_err());
    let file_parent = dir.0.join("file");
    fs::write(&file_parent, b"x").unwrap_or_default();
    assert!(save(&file_parent.join("p.toml"), &profile).is_err());
    assert!(save(Path::new("/"), &profile).is_err());
    // Missing file, but the default cannot be written: still a usable profile.
    let q = dir.0.join("q.toml");
    fs::create_dir_all(q.with_extension("toml.tmp")).unwrap_or_default();
    assert!(ensure_default(&q).is_ok());
}

#[test]
fn loading_never_rewrites_an_existing_profile() {
    // The controller only reads the profile. Runtime state lives in state.toml.
    let dir = TempDir::new("readonly");
    let path = dir.0.join("profile.toml");
    save(&path, &Profile::default_from_legacy()).unwrap_or_default();
    let before = fs::read(&path).unwrap_or_default();
    let mtime_before = modified_at(&path);
    for _ in 0..3 {
        assert_eq!(read_ok(&path), Profile::default_from_legacy());
    }
    assert_eq!(fs::read(&path).unwrap_or_default(), before);
    assert_eq!(modified_at(&path), mtime_before);
    assert_eq!(modified_at(&dir.0.join("gone.toml")), None);
}

#[test]
fn import_handles_missing_names_and_extensionless_files() {
    let root = TempDir::new("imp-root");
    let outside = TempDir::new("imp-out");
    // A file that cannot be read is an error, not an empty copy.
    let locked = outside.0.join("locked.png");
    fs::write(&locked, b"secret").unwrap_or_default();
    let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o000));
    let refused = import_image(&root.0, &locked).is_err();
    let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o644));
    assert!(refused || running_as_root());
    assert!(import_image(&root.0, Path::new("/")).is_err());
    assert!(import_image(&root.0, &outside.0.join("missing.png")).is_err());

    let plain = outside.0.join("icon");
    fs::write(&plain, b"a").unwrap_or_default();
    let first = import_image(&root.0, &plain).unwrap_or_default();
    assert_eq!(first, PathBuf::from("images/icon"));
    fs::write(&plain, b"b").unwrap_or_default();
    assert_eq!(
        import_image(&root.0, &plain).unwrap_or_default(),
        PathBuf::from("images/icon-1")
    );

    // images/ exists as a file, so creating it fails.
    let blocked = TempDir::new("imp-blocked");
    fs::write(blocked.0.join("images"), b"x").unwrap_or_default();
    assert!(import_image(&blocked.0, &plain).is_err());

    // A directory named like the target makes the write fail.
    let clash = TempDir::new("imp-clash");
    fs::create_dir_all(clash.0.join("images/icon")).unwrap_or_default();
    assert!(import_image(&clash.0, &plain).is_err());
}

#[cfg(test)]
#[path = "profile_guide_tests.rs"]
mod guide_tests;
