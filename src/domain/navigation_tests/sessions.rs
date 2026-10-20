use super::*;
use crate::domain::actions::{SessionStatus, TerminalApp};

#[test]
fn set_sessions_replaces_the_list_and_keeps_the_page_in_range() {
    let session = |i: usize| ClaudeSession {
        pid: i as u32,
        app: TerminalApp::Other,
        tty: format!("ttys{i:03}"),
        label: format!("s{i}"),
        status: SessionStatus::Active,
        age: None,
        focused: false,
    };
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::ClaudeSessions {
        sessions: (0..25).map(session).collect(),
        page: 2,
    });
    nav.set_sessions((0..3).map(session).collect());
    assert!(matches!(
        nav.current(),
        Screen::ClaudeSessions { sessions, page: 0 } if sessions.len() == 3
    ));
}

#[test]
fn claude_sessions_page_forward_back_and_stop_at_edges() {
    let sessions: Vec<ClaudeSession> = (0..11)
        .map(|i| ClaudeSession {
            pid: i as u32,
            app: TerminalApp::Other,
            status: SessionStatus::Stale,
            age: None,
            focused: false,
            tty: format!("ttys{i:03}"),
            label: format!("s{i}"),
        })
        .collect();
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::ClaudeSessions { sessions, page: 0 });
    assert!(nav.can_forward());
    nav.forward();
    assert!(!nav.can_forward());
    nav.forward();
    assert!(matches!(
        nav.current(),
        Screen::ClaudeSessions { page: 1, .. }
    ));
    nav.back();
    assert!(matches!(
        nav.current(),
        Screen::ClaudeSessions { page: 0, .. }
    ));
    assert!(nav.can_out());
    nav.previous_page();
    assert!(matches!(
        nav.current(),
        Screen::ClaudeSessions { page: 0, .. }
    ));
    nav.forward();
    nav.previous_page();
    assert!(matches!(
        nav.current(),
        Screen::ClaudeSessions { page: 0, .. }
    ));
}
