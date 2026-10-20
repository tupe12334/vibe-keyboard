use super::Action;
use crate::domain::navigation::Screen;

/// Keys on the Centy project and issue screens.
pub(super) fn action(key: u8, screen: &Screen) -> Action {
    match screen {
        Screen::CentyProjectList { projects, page, .. } => {
            if key == 15 {
                Action::Search
            } else if matches!(key, 1..=10) {
                let idx = page * 10 + (key as usize - 1);
                if projects.get(idx).is_some() {
                    Action::SelectProject {
                        projects: projects.clone(),
                        idx,
                    }
                } else {
                    Action::None
                }
            } else if key == 14 {
                Action::SortList
            } else {
                Action::None
            }
        }
        Screen::CentyProjectActions { project } => match key {
            1 => Action::OpenVsCode {
                name: project.name.clone(),
                path: project.path.clone(),
            },
            2 => Action::OpenTerminalInPath {
                name: project.name.clone(),
                path: project.path.clone(),
            },
            3 => Action::OpenBrowser {
                name: project.name.clone(),
                url: project.url.clone(),
            },
            4 => Action::FetchIssues {
                project_name: project.name.clone(),
                org: project.org.clone(),
            },
            _ => Action::None,
        },
        Screen::CentyIssueList {
            issues,
            page,
            project_name,
            org,
            ..
        } => {
            if key == 15 {
                Action::Search
            } else if matches!(key, 1..=10) {
                let idx = page * 10 + (key as usize - 1);
                if issues.get(idx).is_some() {
                    Action::SelectIssue {
                        issues: issues.clone(),
                        project_name: project_name.clone(),
                        org: org.clone(),
                        idx,
                    }
                } else {
                    Action::None
                }
            } else if key == 14 {
                Action::SortList
            } else {
                Action::None
            }
        }
        Screen::CentyIssueActions {
            issue,
            project_name,
            org,
        } => match key {
            1 => Action::OpenIssueInVsCode {
                file_path: issue.file_path.clone(),
            },
            2 => Action::OpenIssueInWeb {
                id: issue.id.clone(),
                org: org.clone(),
                project_name: project_name.clone(),
            },
            3 => Action::OpenCentyWorkspace {
                issue_number: issue.number,
            },
            _ => Action::None,
        },
        _ => Action::None,
    }
}
