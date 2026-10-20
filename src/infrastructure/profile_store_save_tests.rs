use super::*;

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
