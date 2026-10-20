#![allow(clippy::expect_used)]

use super::*;
use vibe_keyboard::domain::profile::Page;
use vibe_keyboard::infrastructure::profile_store;

/// A fresh profile file with one page, and the path to it.
fn seeded(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("vibe-keyboard-io-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("profile.toml");
    let mut profile = Profile::default();
    profile.add_page("Start");
    profile_store::save(&path, &profile).expect("seed");
    path
}

/// Another editor (or the user) writes the file.
fn external_edit(path: &Path) {
    std::fs::write(path, "[external]\n").expect("external edit");
}

#[test]
fn load_then_save_round_trips_through_the_file() {
    let path = seeded("round");
    let loaded = Mutex::new(None);
    let mut profile = load_layout(&loaded, &path).expect("load");
    profile.add_page("Second");
    save_layout(&loaded, &path, &profile, false).expect("save");
    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages.len(), 2);
}

#[test]
fn force_writes_over_an_external_edit() {
    let path = seeded("force");
    let loaded = Mutex::new(None);
    let mut profile = load_layout(&loaded, &path).expect("load");
    external_edit(&path);
    profile.add_page("Forced");
    save_layout(&loaded, &path, &profile, true).expect("forced save");
    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(
        on_disk.pages.last().map(|p| p.name.as_str()),
        Some("Forced")
    );
}

#[test]
fn a_layout_with_problems_is_refused_before_the_file_is_touched() {
    let path = seeded("problems");
    let before = std::fs::read_to_string(&path).expect("read");
    let loaded = Mutex::new(None);
    let mut profile = load_layout(&loaded, &path).expect("load");
    profile.pages.push(Page::new("   "));
    assert!(matches!(
        save_layout(&loaded, &path, &profile, false),
        Err(SaveError::Refused { .. })
    ));
    assert_eq!(std::fs::read_to_string(&path).expect("read"), before);
}

#[path = "profile_io_conflict_tests.rs"]
mod conflict_tests;
