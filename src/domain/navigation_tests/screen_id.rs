use super::*;
use crate::domain::actions::{SessionStatus, TerminalApp};

fn session() -> ClaudeSession {
    ClaudeSession {
        pid: 1,
        app: TerminalApp::ITerm,
        tty: "ttys001".into(),
        label: "s".into(),
        status: SessionStatus::Active,
        age: None,
        focused: false,
        remote_control: false,
    }
}

#[test]
fn every_screen_change_on_a_session_page_changes_the_id() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::ClaudeSessions {
        sessions: vec![session()],
        page: 0,
    });
    let list = nav.screen_id();
    nav.push(Screen::SessionActions {
        session: session(),
        page: 0,
    });
    let actions = nav.screen_id();
    assert_ne!(list, actions);
    nav.turn_action_page(true, 12);
    let turned = nav.screen_id();
    assert_ne!(actions, turned);
    nav.back();
    assert_ne!(turned, nav.screen_id());
    assert_eq!(list, nav.screen_id());
}

#[test]
fn main_page_turn_changes_the_id() {
    let mut nav = NavigationStack::new(0, 3);
    let first = nav.screen_id();
    nav.go_to_page(1);
    assert_ne!(first, nav.screen_id());
}

#[test]
fn list_screens_have_their_own_ids() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::CentyProjectList {
        projects: Vec::new(),
        page: 0,
        filter: None,
    });
    let projects = nav.screen_id();
    nav.out();
    nav.push(Screen::CentyIssueList {
        issues: Vec::new(),
        page: 0,
        project_name: String::new(),
        org: String::new(),
        filter: None,
    });
    let issues = nav.screen_id();
    nav.out();
    nav.push(Screen::MoadimList {
        list: crate::domain::moadim::MoadimList::Pending,
        page: 0,
    });
    assert_ne!(projects, issues);
    assert_ne!(issues, nav.screen_id());
}

#[test]
fn unpaged_screen_has_an_id() {
    let mut nav = NavigationStack::new(0, 1);
    let main = nav.screen_id();
    nav.push(Screen::NewSessionChoice);
    assert_ne!(main, nav.screen_id());
}
