use super::*;
use crate::domain::actions::SessionStatus;
use crate::domain::navigation::{NavigationStack, Screen};

#[test]
fn selecting_a_session_focuses_its_tab_then_opens_actions() {
    let session = ClaudeSession {
        pid: 1,
        app: TerminalApp::ITerm,
        tty: "ttys009".into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
    };
    let mut nav = NavigationStack::new(0, 1);
    let mut focused = Vec::new();
    open_session_actions(&mut nav, session, |tty| focused.push(tty.to_string()));
    assert_eq!(focused, ["ttys009"]);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 0, .. }
    ));
}
