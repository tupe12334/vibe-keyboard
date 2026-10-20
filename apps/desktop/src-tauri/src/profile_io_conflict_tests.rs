use super::*;
use std::fs::File;
use vibe_keyboard::infrastructure::profile_store;

/// Another editor writes the file and puts the modification time back. Only the bytes differ.
fn external_edit_keeping_time(path: &Path) {
    let before = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .expect("mtime");
    external_edit(path);
    File::options()
        .write(true)
        .open(path)
        .and_then(|f| f.set_modified(before))
        .expect("restore mtime");
}

#[test]
fn an_external_edit_between_load_and_save_is_a_conflict() {
    let path = seeded("external");
    let loaded = Mutex::new(None);
    let profile = load_layout(&loaded, &path).expect("load");
    external_edit(&path);
    assert_eq!(
        save_layout(&loaded, &path, &profile, false),
        Err(SaveError::Conflict)
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "[external]\n"
    );
}

#[test]
fn an_edit_that_keeps_the_modification_time_is_still_a_conflict() {
    let path = seeded("same-time");
    let loaded = Mutex::new(None);
    let profile = load_layout(&loaded, &path).expect("load");
    external_edit_keeping_time(&path);
    assert_eq!(
        save_layout(&loaded, &path, &profile, false),
        Err(SaveError::Conflict)
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "[external]\n"
    );
}

#[test]
fn a_second_window_with_an_old_load_is_a_conflict_after_the_first_saves() {
    let path = seeded("two-windows");
    let first = Mutex::new(None);
    let second = Mutex::new(None);
    let mut a = load_layout(&first, &path).expect("load a");
    let b = load_layout(&second, &path).expect("load b");
    a.add_page("From A");
    save_layout(&first, &path, &a, false).expect("A saves");
    assert_eq!(
        save_layout(&second, &path, &b, false),
        Err(SaveError::Conflict)
    );
    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages.len(), 2);
}
