use super::*;

#[test]
fn a_missing_file_loads_with_no_version() {
    let path = std::env::temp_dir().join("vibe-keyboard-save-no-such-file.toml");
    let _ = std::fs::remove_file(&path);
    let (_, version) = load_with_version(&path).expect("load");
    assert_eq!(version, None);
}

#[test]
fn the_version_follows_the_bytes_not_the_time() {
    let path = temp_profile("bytes");
    let (_, loaded) = load_with_version(&path).expect("load");
    assert_eq!(loaded, version_of(&path));
}

#[test]
fn a_create_only_save_keeps_a_file_another_writer_made() {
    // The deck's startup default expects "missing". A file that appeared since the read stays.
    let path = temp_profile("appeared");
    let profile = Profile::default_from_legacy();
    assert_eq!(
        save_if_unchanged(&path, &profile, None),
        Err(SaveRefusal::Changed)
    );
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "");
}

#[test]
fn a_directory_in_place_of_the_file_fails_to_load() {
    assert!(load_with_version(&std::env::temp_dir()).is_err());
}
