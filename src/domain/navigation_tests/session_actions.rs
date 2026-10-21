use super::*;
use crate::domain::actions::{SessionStatus, TerminalApp};

#[test]
fn prompt_keys_page_through_eight_prompts_each() {
    assert_eq!(prompt_index(0, 1, 3), Some(0));
    assert_eq!(prompt_index(0, 3, 3), Some(2));
    assert_eq!(prompt_index(0, 4, 3), None);
    assert_eq!(prompt_index(0, 10, 20), None);
    assert_eq!(prompt_index(0, 11, 3), None);
    assert_eq!(prompt_index(0, 12, 3), None);
    assert_eq!(prompt_index(1, 1, 12), Some(8));
    assert_eq!(prompt_index(1, 8, 16), Some(15));
    assert_eq!(prompt_index(1, 3, 12), Some(10));
    assert_eq!(prompt_index(1, 5, 12), None);
    assert_eq!(action_pages(0), 1);
    assert_eq!(action_pages(8), 1);
    assert_eq!(action_pages(9), 2);
}

#[test]
fn full_screen_and_blink_keys_are_never_prompt_keys() {
    for page in 0..3 {
        assert_eq!(prompt_index(page, FULL_SCREEN_KEY, 30), None);
        assert_eq!(prompt_index(page, BLINK_KEY, 30), None);
    }
    assert_eq!(BLINK_KEY, 10);
}

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
