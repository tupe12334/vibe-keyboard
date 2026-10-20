use super::*;

#[test]
fn tty_must_be_ttys_and_digits() {
    assert!(is_valid_tty("ttys003"));
    assert!(is_valid_tty("ttys0"));
    assert!(!is_valid_tty("ttys"));
    assert!(!is_valid_tty("ttys00a"));
    assert!(!is_valid_tty("ttys003\" & do shell script \"x"));
    assert!(!is_valid_tty("/dev/ttys003"));
    assert!(!is_valid_tty("??"));
    assert!(!is_valid_tty(""));
}

#[test]
fn is_session_on_tty_needs_the_same_pid_and_tty() {
    let ps = "  101 ttys003 claude\n  202 ttys004 claude\n";
    assert!(is_session_on_tty(ps, 101, "ttys003"));
    assert!(!is_session_on_tty(ps, 101, "ttys004"), "pid moved tty");
    assert!(!is_session_on_tty(ps, 999, "ttys003"), "pid gone");
    assert!(!is_session_on_tty("", 101, "ttys003"));
}
