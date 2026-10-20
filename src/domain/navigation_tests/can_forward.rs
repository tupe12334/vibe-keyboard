use super::support::{make_issue, make_issues, make_project, make_projects};
use super::*;

#[test]
fn can_forward_main_page_always_true() {
    let nav = NavigationStack::new(0, 2);
    assert!(nav.can_forward());
}

#[test]
fn can_forward_project_list_false_when_at_last_page() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectList {
        projects: make_projects(5),
        page: 0,
        filter: None,
    });
    assert!(!nav.can_forward());
}

#[test]
fn can_forward_project_list_true_when_more_pages() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectList {
        projects: make_projects(15),
        page: 0,
        filter: None,
    });
    assert!(nav.can_forward());
}

#[test]
fn can_forward_issue_list_false_when_at_last_page() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueList {
        issues: make_issues(5),
        page: 0,
        project_name: "p".into(),
        org: "org".into(),
        filter: None,
    });
    assert!(!nav.can_forward());
}

#[test]
fn can_forward_issue_list_true_when_more_pages() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueList {
        issues: make_issues(15),
        page: 0,
        project_name: "p".into(),
        org: "org".into(),
        filter: None,
    });
    assert!(nav.can_forward());
}

#[test]
fn can_forward_project_actions_false() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectActions {
        project: make_project(),
    });
    assert!(!nav.can_forward());
}

#[test]
fn can_forward_issue_actions_false() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueActions {
        issue: make_issue(),
        project_name: "p".into(),
        org: "org".into(),
    });
    assert!(!nav.can_forward());
}
