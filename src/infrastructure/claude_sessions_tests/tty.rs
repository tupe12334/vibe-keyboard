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
fn live_pid_on_its_tty_is_unchanged() {
    let ps = "  101 ttys003 claude\n  202 ttys004 claude\n";
    assert_eq!(resolve_session_pid(ps, 101, "ttys003"), Some(101));
}

#[test]
fn stale_pid_resolves_to_the_claude_now_on_the_tty() {
    // Session on ttys003 restarted as pid 303. Pid 101 is gone.
    let ps = "  303 ttys003 claude\n  202 ttys004 claude\n";
    assert_eq!(resolve_session_pid(ps, 101, "ttys003"), Some(303));
}

#[test]
fn stale_pid_with_no_claude_on_the_tty_is_none() {
    let ps = "  202 ttys004 claude\n  404 ttys003 zsh\n";
    assert_eq!(resolve_session_pid(ps, 101, "ttys003"), None);
    assert_eq!(resolve_session_pid("", 101, "ttys003"), None);
}

#[test]
fn pid_that_moved_tty_resolves_to_the_claude_on_its_own_tty() {
    let ps = "  101 ttys004 claude\n  303 ttys003 claude\n";
    assert_eq!(resolve_session_pid(ps, 101, "ttys003"), Some(303));
}
