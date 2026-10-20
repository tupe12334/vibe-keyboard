use super::support::{make_issue, make_project};
use super::*;

#[test]
fn forward_project_actions_noop() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectActions {
        project: make_project(),
    });
    nav.forward();
    assert!(matches!(nav.current(), Screen::CentyProjectActions { .. }));
}

#[test]
fn forward_issue_actions_noop() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueActions {
        issue: make_issue(),
        project_name: "p".into(),
        org: "org".into(),
    });
    nav.forward();
    assert!(matches!(nav.current(), Screen::CentyIssueActions { .. }));
}
