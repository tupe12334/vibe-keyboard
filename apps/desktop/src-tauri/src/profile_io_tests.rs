#![allow(clippy::expect_used)]

use super::*;
use std::fs::File;
use std::time::Duration;
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

/// Another editor (or the user) writes the file and its time moves forward.
fn external_edit(path: &Path) {
    std::fs::write(path, "[external]\n").expect("external edit");
    let later = SystemTime::now() + Duration::from_secs(60);
    File::options()
        .write(true)
        .open(path)
        .and_then(|f| f.set_modified(later))
        .expect("set mtime");
}

#[test]
fn load_then_save_round_trips_through_the_file() {
    let path = seeded("round");
    let loaded = Mutex::new(None);
    let mut profile = load_layout(&loaded, &path).expect("load");
    profile.add_page("Second");
    save_layout(&loaded, &path, &profile).expect("save");
    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages.len(), 2);
}

#[test]
fn an_external_edit_between_load_and_save_is_refused() {
    let path = seeded("external");
    let loaded = Mutex::new(None);
    let profile = load_layout(&loaded, &path).expect("load");
    external_edit(&path);
    let refused = save_layout(&loaded, &path, &profile);
    assert!(refused.expect_err("refused").contains("changed on disk"));
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "[external]\n"
    );
}

#[test]
fn a_second_window_with_an_old_load_is_refused_after_the_first_saves() {
    let path = seeded("two-windows");
    let first = Mutex::new(None);
    let second = Mutex::new(None);
    let mut a = load_layout(&first, &path).expect("load a");
    let b = load_layout(&second, &path).expect("load b");
    // Window A saves a real change, which moves the file time.
    a.add_page("From A");
    save_layout(&first, &path, &a).expect("A saves");
    // Window B still holds the old time. Its save is refused, and A's layout stays on disk.
    let refused = save_layout(&second, &path, &b);
    assert!(refused.expect_err("B refused").contains("changed on disk"));
    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages.len(), 2);
}

#[test]
fn a_layout_with_problems_is_refused_before_the_file_is_touched() {
    let path = seeded("problems");
    let before = std::fs::read_to_string(&path).expect("read");
    let loaded = Mutex::new(None);
    let mut profile = load_layout(&loaded, &path).expect("load");
    profile.pages.push(Page::new("   "));
    assert!(save_layout(&loaded, &path, &profile).is_err());
    assert_eq!(std::fs::read_to_string(&path).expect("read"), before);
}
