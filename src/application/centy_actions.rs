use std::sync::{Arc, Mutex};

use tracing::info;

use crate::domain::navigation::{NavigationStack, Screen};
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

use super::actions::{
    open_centy_workspace, open_in_chrome, open_terminal_in_path, open_vscode_in_path,
};
use super::key_action::Action;
use super::pages::fetch_centy_issues;
use super::{lock, render_screen, DeviceState};

/// Centy project and issue keys. Each opens a page, a folder, the browser or a workspace.
pub(super) fn centy_action(
    action: Action,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    match action {
        Action::SelectProject { projects, idx } => {
            if let Some(project) = projects.into_iter().nth(idx) {
                info!("centy: selected project {}", project.name);
                nav.push(Screen::CentyProjectActions { project });
                render_screen(nav, handle, state, dev_state);
            }
        }
        Action::OpenVsCode { name, path } => {
            info!("centy: open {} in VS Code", name);
            open_vscode_in_path(path.as_deref().unwrap_or("."));
        }
        Action::OpenTerminalInPath { name, path } => {
            info!("centy: open {} in Terminal", name);
            open_terminal_in_path(path.as_deref());
        }
        Action::OpenBrowser { name, url } => {
            info!("centy: open {} in browser", name);
            open_in_chrome(&url);
        }
        Action::FetchIssues { project_name, org } => {
            lock(state).loading = true;
            let issues = fetch_centy_issues(&project_name);
            lock(state).loading = false;
            if !issues.is_empty() {
                nav.push(Screen::CentyIssueList {
                    issues,
                    page: 0,
                    project_name,
                    org,
                    filter: None,
                });
                render_screen(nav, handle, state, dev_state);
            }
        }
        Action::SelectIssue {
            issues,
            project_name,
            org,
            idx,
        } => {
            if let Some(issue) = issues.into_iter().nth(idx) {
                info!("centy: selected issue #{}", issue.number);
                nav.push(Screen::CentyIssueActions {
                    issue,
                    project_name,
                    org,
                });
                render_screen(nav, handle, state, dev_state);
            }
        }
        Action::OpenIssueInVsCode { file_path } => {
            let path = file_path.as_deref().unwrap_or(".");
            info!("centy: open issue in VS Code at {}", path);
            open_vscode_in_path(path);
        }
        Action::OpenIssueInWeb {
            id,
            org,
            project_name,
        } => {
            let url = format!("https://app.centy.io/{org}/{project_name}/issues/{id}");
            info!("centy: open issue in web: {}", url);
            open_in_chrome(&url);
        }
        Action::OpenCentyWorkspace { issue_number } => {
            info!("centy: opening workspace for issue {}", issue_number);
            open_centy_workspace(issue_number);
        }
        _ => {}
    }
}
