use super::must_ask;

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
