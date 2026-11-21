use tracing::info;

use super::issues_json::parse_centy_issues_json;
use super::projects::CENTY_TIMEOUT;
use crate::domain::actions::CentyIssue;
use crate::infrastructure::process::probe;

pub fn fetch_centy_issues(project_name: &str) -> Vec<CentyIssue> {
    info!("running centy list issues for {}", project_name);

    let Some(stdout) = probe(
        "pnpm",
        &[
            "dlx",
            "centy",
            "list",
            "issues",
            "--project",
            project_name,
            "--json",
        ],
        CENTY_TIMEOUT,
    ) else {
        info!(
            "centy: no issues output, failed, or timed out for {}",
            project_name
        );
        return vec![];
    };
    let issues = parse_centy_issues_json(&stdout);

    if issues.is_empty() {
        info!("centy: no issues found for {}", project_name);
    } else {
        info!("centy: {} issue(s) for {}", issues.len(), project_name);
    }

    issues
}
