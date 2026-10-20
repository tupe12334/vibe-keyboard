use std::sync::{Arc, Mutex};

use tracing::info;

use super::render_screen;
use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

pub fn handle_filter_query(
    query: &str,
    handle: &Device,
    nav: &mut NavigationStack,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    use crate::domain::actions::{CentyIssue, CentyProject};

    enum FilterAction {
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

    match fa {
        FilterAction::Projects { projects, filter } => {
            info!(
                "filter applied: {} project(s) match {:?}",
                projects.len(),
                query
            );
            nav.push(Screen::CentyProjectList {
                projects,
                page: 0,
                filter,
            });
            render_screen(nav, handle, state, dev_state);
        }
        FilterAction::Issues {
            issues,
            project_name,
            org,
            filter,
        } => {
            info!(
                "filter applied: {} issue(s) match {:?}",
                issues.len(),
                query
            );
            nav.push(Screen::CentyIssueList {
                issues,
                page: 0,
                project_name,
                org,
                filter,
            });
            render_screen(nav, handle, state, dev_state);
        }
        FilterAction::None => {}
    }
}
