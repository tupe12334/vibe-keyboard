use super::{newest, opens_on};
use crate::domain::actions::{ClaudeSession, SessionStatus, TerminalApp};
use crate::domain::navigation::{NavigationStack, Screen};

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

#[test]
fn opens_only_on_the_sessions_list() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::ClaudeSessions {
        sessions: vec![session("ttys001")],
        page: 0,
    });
    assert!(opens_on(&nav));
    nav.push(Screen::SessionActions {
        session: session("ttys001"),
        page: 0,
    });
    assert!(!opens_on(&nav));
}
