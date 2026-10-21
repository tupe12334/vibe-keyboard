#![allow(clippy::expect_used)]

use super::*;

#[test]
fn a_long_message_is_cut_to_500_characters() {
    let line = webview_log_line("error", &"a".repeat(600));
    assert_eq!(line, format!("editor webview: error: {}", "a".repeat(500)));
}

#[test]
fn the_cut_counts_characters_not_bytes() {
    let line = webview_log_line("error", &"é".repeat(600));
    assert_eq!(line.chars().count(), "editor webview: error: ".len() + 500);
}

#[test]
fn line_breaks_become_spaces() {
    let line = webview_log_line("csp", "a\nb\r\nc");
    assert_eq!(line, "editor webview: csp: a b  c");
    assert!(!line.contains('\n') && !line.contains('\r'));
}

#[test]
fn a_saved_profile_line_counts_keys_on_every_page() {
    let mut profile = Profile::default();
    profile.add_page("First");
    profile.add_page("Second");
    profile.pages[0].keys.insert(0, Default::default());
    profile.pages[0].keys.insert(1, Default::default());
    profile.pages[1].keys.insert(0, Default::default());
    assert_eq!(
        saved_line(&profile, "/x/profile.toml"),
        "editor: saved profile (3 keys) to /x/profile.toml"
    );
}

#[test]
fn a_failed_save_line_has_no_line_breaks() {
    assert_eq!(
        save_failed_line("disk full\nretry"),
        "editor: save failed: disk full retry"
    );
}
