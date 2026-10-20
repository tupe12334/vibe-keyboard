use super::*;

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
