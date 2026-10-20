use super::newest;
use crate::domain::actions::{ClaudeSession, SessionStatus, TerminalApp};

fn session(tty: &str) -> ClaudeSession {
    ClaudeSession {
        pid: 1,
        app: TerminalApp::ITerm,
        tty: tty.into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
    }
}

#[test]
fn newest_is_the_session_on_a_new_tty() {
    let before = vec!["ttys001".to_string()];
    assert!(newest(vec![session("ttys001")], &before).is_none());
    let found = newest(vec![session("ttys001"), session("ttys004")], &before);
    assert_eq!(found.map(|s| s.tty), Some("ttys004".to_string()));
}
