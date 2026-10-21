use super::*;
use crate::domain::actions::{SessionStatus, TerminalApp};

#[test]
fn action_page_turns_within_its_range() {
    let session = ClaudeSession {
        pid: 1,
        app: TerminalApp::ITerm,
        tty: "ttys001".into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
        remote_control: false,
    };
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::SessionActions { session, page: 0 });
    nav.turn_action_page(true, 12);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 1, .. }
    ));
    // Forward wraps from the last page to the first.
    nav.turn_action_page(true, 12);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 0, .. }
    ));
    nav.turn_action_page(true, 12);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 1, .. }
    ));
    nav.turn_action_page(false, 12);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 0, .. }
    ));
    // Other screens ignore the turn.
    let mut main = NavigationStack::new(0, 2);
    main.turn_action_page(true, 12);
    assert!(matches!(main.current(), Screen::MainPage { page: 0 }));
}

#[test]
fn open_action_page_clamps_when_prompts_shrink() {
    let session = ClaudeSession {
        pid: 1,
        app: TerminalApp::ITerm,
        tty: "ttys001".into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
        remote_control: false,
    };
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::SessionActions { session, page: 1 });
    nav.set_action_prompts(11);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 1, .. }
    ));
    nav.set_action_prompts(8);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 0, .. }
    ));
    nav.set_action_prompts(0);
    assert!(matches!(
        nav.current(),
        Screen::SessionActions { page: 0, .. }
    ));
}
