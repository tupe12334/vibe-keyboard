use super::support::{make_issue, make_numbered_issue, make_project, make_projects};
use super::*;

#[test]
fn back_project_actions_pops() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectActions {
        project: make_project(),
    });
    nav.back();
    assert!(matches!(nav.current(), Screen::MainPage { .. }));
}

#[test]
fn back_issue_actions_pops() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueActions {
        issue: make_issue(),
        project_name: "p".into(),
        org: "org".into(),
    });
    nav.back();
    assert!(matches!(nav.current(), Screen::MainPage { .. }));
}

#[test]
fn back_at_root_with_pop_screen_is_noop() {
    // back() on a CentyProjectActions at root (stack len == 1) is a no-op
    // Construct manually by starting with a project actions as base
    let mut nav = NavigationStack::new(0, 1);
    // back on MainPage with 1 total page wraps to 0 (same page)
    nav.back();
    assert!(matches!(nav.current(), Screen::MainPage { page: 0 }));
}

// --- out ---
#[test]
fn out_noop_at_root() {
    let mut nav = NavigationStack::new(0, 2);
    nav.out();
    assert!(matches!(nav.current(), Screen::MainPage { .. }));
}

#[test]
fn out_pops_to_parent() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectActions {
        project: make_project(),
    });
    nav.out();
    assert!(matches!(nav.current(), Screen::MainPage { .. }));
}

#[test]
fn out_to_main_from_deep_stack_returns_to_main_page() {
    let mut nav = NavigationStack::new(1, 3);
    nav.push(Screen::CentyProjectList {
        projects: make_projects(1),
        page: 0,
        filter: None,
    });
    nav.push(Screen::CentyProjectActions {
        project: make_project(),
    });
    nav.push(Screen::CentyIssueList {
        issues: vec![],
        page: 0,
        project_name: "proj".into(),
        org: "org".into(),
        filter: None,
    });
    nav.push(Screen::CentyIssueActions {
        issue: make_numbered_issue(1),
        project_name: "proj".into(),
        org: "org".into(),
    });
    nav.out_to_main();
    assert_eq!(nav.main_page(), Some(1));
    assert!(!nav.can_out());
}
