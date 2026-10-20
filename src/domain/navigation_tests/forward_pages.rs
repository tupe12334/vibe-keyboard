use super::support::{make_issues, make_projects};
use super::*;

// --- forward ---
#[test]
fn forward_main_page_wraps() {
    let mut nav = NavigationStack::new(1, 2);
    nav.forward();
    assert!(matches!(nav.current(), Screen::MainPage { page: 0 }));
}

#[test]
fn forward_main_page_increments() {
    let mut nav = NavigationStack::new(0, 2);
    nav.forward();
    assert!(matches!(nav.current(), Screen::MainPage { page: 1 }));
}

#[test]
fn forward_project_list_increments() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectList {
        projects: make_projects(15),
        page: 0,
        filter: None,
    });
    nav.forward();
    assert!(matches!(
        nav.current(),
        Screen::CentyProjectList { page: 1, .. }
    ));
}

#[test]
fn forward_project_list_noop_at_max() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectList {
        projects: make_projects(5),
        page: 0,
        filter: None,
    });
    nav.forward();
    assert!(matches!(
        nav.current(),
        Screen::CentyProjectList { page: 0, .. }
    ));
}

#[test]
fn forward_issue_list_increments() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueList {
        issues: make_issues(15),
        page: 0,
        project_name: "p".into(),
        org: "org".into(),
        filter: None,
    });
    nav.forward();
    assert!(matches!(
        nav.current(),
        Screen::CentyIssueList { page: 1, .. }
    ));
}

#[test]
fn forward_issue_list_noop_at_max() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueList {
        issues: make_issues(5),
        page: 0,
        project_name: "p".into(),
        org: "org".into(),
        filter: None,
    });
    nav.forward();
    assert!(matches!(
        nav.current(),
        Screen::CentyIssueList { page: 0, .. }
    ));
}
