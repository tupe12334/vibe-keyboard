use super::*;
use crate::domain::actions::{ClaudeSession, SessionStatus};

fn session(app: TerminalApp, tty: &str) -> ClaudeSession {
    ClaudeSession {
        pid: 1,
        app,
        tty: tty.into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
        remote_control: false,
    }
}

/// The sessions list with one session's action page open on top.
fn on_actions(app: TerminalApp, tty: &str) -> NavigationStack {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::ClaudeSessions {
        sessions: vec![session(app, tty)],
        page: 0,
    });
    nav.push(Screen::SessionActions {
        session: session(app, tty),
        page: 0,
    });
    nav
}

#[test]
fn prompt_send_success_leaves_to_the_sessions_list() {
    let mut nav = on_actions(TerminalApp::ITerm, "ttys009");
    assert!(leave_after_send(&nav, "ttys009", true));
    nav.back();
    assert!(matches!(nav.current(), Screen::ClaudeSessions { .. }));
}

#[test]
fn prompt_send_failure_stays_on_the_action_page() {
    let nav = on_actions(TerminalApp::ITerm, "ttys009");
    assert!(!leave_after_send(&nav, "ttys009", false));
}

#[test]
fn muted_app_never_sends_so_it_stays() {
    let nav = on_actions(TerminalApp::Other, "ttys009");
    assert!(!may_send(TerminalApp::Other));
    assert!(matches!(nav.current(), Screen::SessionActions { .. }));
}

#[test]
fn voice_hold_never_leaves_the_action_page() {
    // Key 13 starts no send, so no delivered result can arrive for it. Without a result the page
    // stays, whatever the send state.
    let nav = on_actions(TerminalApp::Terminal, "ttys009");
    assert!(!leave_after_send(&nav, "ttys009", false));
}

#[test]
fn a_result_for_another_session_stays() {
    let nav = on_actions(TerminalApp::ITerm, "ttys009");
    assert!(!leave_after_send(&nav, "ttys010", true));
}

#[test]
fn a_user_who_already_left_stays_put() {
    let mut nav = on_actions(TerminalApp::ITerm, "ttys009");
    nav.back();
    assert!(!leave_after_send(&nav, "ttys009", true));
}
