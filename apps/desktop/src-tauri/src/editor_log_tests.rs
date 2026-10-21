#![allow(clippy::expect_used)]

use super::*;
use vibe_keyboard::domain::profile::{Action, KeyConfig};
use vibe_keyboard::domain::profile_rules;

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

#[test]
fn tab_escape_and_nul_in_the_message_become_spaces() {
    let line = webview_log_line("error", "a\tb\u{1b}[31mc\0d");
    assert_eq!(line, "editor webview: error: a b [31mc d");
    assert!(line.chars().all(|c| !c.is_control()));
}

#[test]
fn an_over_long_kind_is_cut_to_40_characters() {
    let line = webview_log_line(&"k".repeat(200), "boom");
    assert_eq!(line, format!("editor webview: {}: boom", "k".repeat(40)));
}

#[test]
fn a_refused_save_logs_the_count_not_the_problem_text() {
    let mut profile = Profile::default();
    profile.add_page("Main");
    profile.pages[0].keys.insert(
        1,
        KeyConfig {
            action: Action::Builtin {
                name: "SECRET-cmd-123".into(),
            },
            ..Default::default()
        },
    );
    let problems = profile_rules::save_problems(&profile);
    assert!(
        problems.iter().any(|p| p.contains("SECRET-cmd-123")),
        "the problem text carries the payload, so the test is meaningful"
    );
    let line = save_refused_line(problems.len());
    assert!(!line.contains("SECRET"), "payload leaked: {line}");
    assert_eq!(
        line,
        format!(
            "editor: save refused: {} problem(s), details not logged",
            problems.len()
        )
    );
}
