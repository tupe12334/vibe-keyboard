use super::must_ask;

#[test]
fn a_quit_waits_for_the_editor_only_when_unconfirmed_and_open() {
    assert!(must_ask(false, true));
    assert!(!must_ask(true, true), "a confirmed quit must not ask again");
    assert!(!must_ask(false, false), "no editor to ask: quit now");
}
