use super::*;

pub fn make_project() -> CentyProject {
    CentyProject {
        name: "proj".into(),
        org: "org".into(),
        path: None,
        url: "https://example.com".into(),
    }
}

pub fn make_issue() -> CentyIssue {
    CentyIssue {
        number: 1,
        title: "issue".into(),
        status: "open".into(),
        id: "abc".into(),
        file_path: None,
    }
}

pub fn make_projects(n: usize) -> Vec<CentyProject> {
    (0..n).map(|_| make_project()).collect()
}

pub fn make_issues(n: usize) -> Vec<CentyIssue> {
    (0..n).map(|_| make_issue()).collect()
}

pub fn make_named_project(name: &str) -> CentyProject {
    CentyProject {
        name: name.into(),
        org: "org".into(),
        path: None,
        url: "https://example.com".into(),
    }
}

pub fn make_numbered_issue(number: u64) -> CentyIssue {
    CentyIssue {
        number,
        title: "issue".into(),
        status: "open".into(),
        id: "abc".into(),
        file_path: None,
    }
}

pub fn current_project_names(nav: &NavigationStack) -> Vec<String> {
    match nav.current() {
        Screen::CentyProjectList { projects, .. } => {
            projects.iter().map(|p| p.name.clone()).collect()
        }
        _ => panic!("expected CentyProjectList"),
    }
}

pub fn current_issue_numbers(nav: &NavigationStack) -> Vec<u64> {
    match nav.current() {
        Screen::CentyIssueList { issues, .. } => issues.iter().map(|i| i.number).collect(),
        _ => panic!("expected CentyIssueList"),
    }
}

pub fn routine(i: usize) -> Routine {
    Routine {
        id: format!("id-{i}"),
        title: format!("r{i}"),
        enabled: true,
        running: false,
        failures: 0,
        last_trigger: None,
    }
}

pub fn online(n: usize) -> MoadimList {
    MoadimList::Online((0..n).map(routine).collect())
}
