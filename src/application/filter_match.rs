//! Picks the rows of the current list screen that match a filter query.

use crate::domain::actions::{CentyIssue, CentyProject};
use crate::domain::navigation::{NavigationStack, Screen};

pub(super) enum FilterAction {
    Projects {
        projects: Vec<CentyProject>,
        filter: Option<String>,
    },
    Issues {
        issues: Vec<CentyIssue>,
        project_name: String,
        org: String,
        filter: Option<String>,
    },
    None,
}

pub(super) fn filter_action(nav: &NavigationStack, query: &str) -> FilterAction {
    let q = query.to_lowercase();
    let filter = if q.trim().is_empty() {
        None
    } else {
        Some(query.to_string())
    };

    let fa = match nav.current() {
        Screen::CentyProjectList { projects, .. } => {
            let filtered: Vec<CentyProject> = if q.trim().is_empty() {
                projects.clone()
            } else {
                projects
                    .iter()
                    .filter(|p| {
                        p.name.to_lowercase().contains(&q) || p.org.to_lowercase().contains(&q)
                    })
                    .cloned()
                    .collect()
            };
            FilterAction::Projects {
                projects: filtered,
                filter,
            }
        }
        Screen::CentyIssueList {
            issues,
            project_name,
            org,
            ..
        } => {
            let filtered: Vec<CentyIssue> = if q.trim().is_empty() {
                issues.clone()
            } else {
                issues
                    .iter()
                    .filter(|i| {
                        i.title.to_lowercase().contains(&q)
                            || i.number.to_string().contains(&q)
                            || i.status.to_lowercase().contains(&q)
                    })
                    .cloned()
                    .collect()
            };
            FilterAction::Issues {
                issues: filtered,
                project_name: project_name.clone(),
                org: org.clone(),
                filter,
            }
        }
        _ => FilterAction::None,
    };
    fa
}
