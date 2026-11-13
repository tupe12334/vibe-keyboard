use super::*;

#[test]
fn gate_retries_then_refuses_when_no_terminal_is_in_front() {
    let mut reads = 0;
    let mut pauses = 0;
    let never = first_landed_pid(
        4,
        || {
            reads += 1;
            None
        },
        || pauses += 1,
    );
    assert_eq!(never, None, "a wrong app in front posts no key");
    assert_eq!((reads, pauses), (4, 3));
}

#[test]
fn gate_takes_the_first_pid_that_lands() {
    let mut calls = 0;
    let third = first_landed_pid(
        4,
        || {
            calls += 1;
            (calls == 3).then_some(42)
        },
        || {},
    );
    assert_eq!(third, Some(42));
}

#[test]
fn only_the_two_terminal_bundles_pass_the_gate() {
    assert_eq!(terminal_index(Some("com.googlecode.iterm2")), Some(0));
    assert_eq!(terminal_index(Some("com.apple.Terminal")), Some(1));
    assert_eq!(terminal_index(Some("com.google.Chrome")), None);
    assert_eq!(terminal_index(None), None);
}
