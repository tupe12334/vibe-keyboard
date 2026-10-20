use super::*;

#[test]
fn discard_is_refused_when_clean() {
    let mut session = Session::new(Profile::default_from_legacy());
    let now = Instant::now();
    assert!(!session.dirty());
    assert!(!session.discard(Profile::default_from_legacy(), now));
    assert!(!session.undo_available(now));
}

#[test]
fn dirty_then_discard_then_undo_returns_to_dirty() {
    let disk = Profile::default_from_legacy();
    let mut session = Session::new(disk.clone());
    session.profile = edited_layout();
    let now = Instant::now();
    assert!(session.dirty());

    assert!(session.discard(disk.clone(), now));
    assert!(!session.dirty());
    assert_eq!(session.profile, disk);
    assert!(session.undo_available(now));

    assert!(session.undo(now));
    assert!(session.dirty());
    assert_eq!(session.profile, edited_layout());
    assert!(!session.undo_available(now));
}

#[test]
fn undo_expires_after_the_window() {
    let disk = Profile::default_from_legacy();
    let mut session = Session::new(disk.clone());
    session.profile = edited_layout();
    let start = Instant::now();
    assert!(session.discard(disk.clone(), start));
    let later = start + UNDO_WINDOW + Duration::from_millis(1);
    assert!(!session.undo_available(later));
    session.expire(later);
    assert!(!session.undo(later));
    assert_eq!(session.profile, disk);
}

#[test]
fn save_clears_the_undo() {
    let disk = Profile::default_from_legacy();
    let mut session = Session::new(disk.clone());
    session.profile = edited_layout();
    let now = Instant::now();
    assert!(session.discard(disk, now));
    session.mark_saved();
    assert!(!session.undo_available(now));
}

#[test]
fn a_single_key_edit_makes_the_session_dirty() {
    let mut session = Session::new(Profile::default_from_legacy());
    session.profile.set_key(
        0,
        7,
        KeyConfig {
            label: "x".into(),
            ..KeyConfig::default()
        },
    );
    assert!(session.dirty());
}
