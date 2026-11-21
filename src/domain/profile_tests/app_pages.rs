use super::*;

#[test]
fn page_for_app_picks_first_match() {
    let mut profile = Profile::default_from_legacy();
    profile.pages[1].apps = vec!["com.google.Chrome".into()];
    profile.pages[0].apps = vec!["com.google.Chrome".into()];
    assert_eq!(profile.page_for_app("com.google.Chrome"), Some(0));
    assert_eq!(profile.page_for_app("com.apple.Notes"), None);
}

#[test]
fn bundle_id_detection() {
    assert!(is_bundle_id("com.apple.Terminal"));
    assert!(!is_bundle_id("Terminal"));
    assert!(!is_bundle_id("My App.app"));
    assert!(!is_bundle_id("Chrome.app"));
    assert!(!is_bundle_id("Chrome.App"));
    assert!(!is_bundle_id("Chrome.APP"));
}
