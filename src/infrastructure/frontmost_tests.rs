use super::{detect_change, parse_bundle_id};

#[test]
fn parses_lsappinfo_bundle_line() {
    assert_eq!(
        parse_bundle_id("\"CFBundleIdentifier\"=\"com.googlecode.iterm2\"\n"),
        Some("com.googlecode.iterm2".into())
    );
}

#[test]
fn rejects_missing_or_empty_bundle_line() {
    assert_eq!(parse_bundle_id(""), None);
    assert_eq!(parse_bundle_id("[ NULL ]"), None);
    assert_eq!(parse_bundle_id("\"CFBundleIdentifier\"=\"\""), None);
}

#[test]
fn first_value_is_a_change() {
    let mut last = None;
    assert_eq!(
        detect_change(&mut last, Some("com.a".into())),
        Some("com.a".into())
    );
}

#[test]
fn same_value_is_not_a_change() {
    let mut last = Some("com.a".into());
    assert_eq!(detect_change(&mut last, Some("com.a".into())), None);
}

#[test]
fn new_value_is_a_change_and_updates_last() {
    let mut last = Some("com.a".into());
    assert_eq!(
        detect_change(&mut last, Some("com.b".into())),
        Some("com.b".into())
    );
    assert_eq!(last.as_deref(), Some("com.b"));
}

#[test]
fn none_is_ignored_and_keeps_last() {
    let mut last = Some("com.a".into());
    assert_eq!(detect_change(&mut last, None), None);
    assert_eq!(last.as_deref(), Some("com.a"));
}

#[test]
fn new_app_after_none_is_a_change() {
    let mut last = Some("com.a".into());
    assert_eq!(detect_change(&mut last, None), None);
    assert_eq!(
        detect_change(&mut last, Some("com.b".into())),
        Some("com.b".into())
    );
}
