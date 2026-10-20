use super::*;

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
