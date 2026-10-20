use super::{is_repeat, must_ask, REPEAT_WINDOW};
use std::time::{Duration, Instant};

#[test]
fn a_quit_waits_for_the_editor_only_when_unconfirmed_and_loaded() {
    assert!(must_ask(false, true, true));
    assert!(
        !must_ask(true, true, true),
        "a confirmed quit must not ask again"
    );
    assert!(!must_ask(false, false, true), "no editor to ask: quit now");
}

#[test]
fn a_quit_before_the_editor_loads_goes_ahead() {
    // The editor has not asked for the layout, so it has no edits: quit without asking.
    assert!(!must_ask(false, true, false));
}

#[test]
fn a_repeat_quit_exits_only_when_the_first_went_unanswered_within_the_window() {
    let asked = Instant::now();
    assert!(!is_repeat(None, asked), "no unanswered request: ask");
    assert!(is_repeat(
        Some(asked),
        asked + REPEAT_WINDOW - Duration::from_secs(1)
    ));
    assert!(!is_repeat(
        Some(asked),
        asked + REPEAT_WINDOW + Duration::from_secs(1)
    ));
}
