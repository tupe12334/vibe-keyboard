use super::*;

#[test]
fn selected_tty_matches_its_device_path_only() {
    assert!(is_selected(Some("/dev/ttys001\n"), "ttys001"));
    assert!(!is_selected(Some("/dev/ttys002"), "ttys001"));
    assert!(!is_selected(None, "ttys001"));
}

#[test]
fn session_moves_only_to_a_known_other_tty() {
    assert!(session_moved(Some("/dev/ttys002"), "ttys001"));
    assert!(!session_moved(Some("/dev/ttys001"), "ttys001"));
    assert!(!session_moved(Some("  "), "ttys001"), "blank is unknown");
    assert!(!session_moved(None, "ttys001"), "unknown is not moved");
}

#[test]
fn script_index_maps_to_its_terminal_bundle() {
    assert_eq!(focused_bundle(0), Some("com.googlecode.iterm2"));
    assert_eq!(focused_bundle(1), Some("com.apple.Terminal"));
    assert_eq!(focused_bundle(2), None);
}

#[test]
fn frontmost_must_match_the_expected_bundle() {
    assert!(is_frontmost(Some("com.apple.Terminal"), TERMINAL_BUNDLE));
    assert!(!is_frontmost(Some("com.google.Chrome"), TERMINAL_BUNDLE));
    assert!(
        !is_frontmost(None, TERMINAL_BUNDLE),
        "unknown front is no match"
    );
}

#[test]
fn scan_result_separates_missing_from_failed() {
    assert_eq!(scan_result(Some("found\n")), Scan::Found);
    assert_eq!(scan_result(Some("missing")), Scan::Missing);
    assert_eq!(
        scan_result(Some("")),
        Scan::Failed,
        "osascript error prints nothing"
    );
    assert_eq!(scan_result(None), Scan::Failed);
}
