use std::process::Command;
use tracing::info;

use super::issues_json::parse_centy_issues_json;
use crate::domain::actions::CentyIssue;

pub fn fetch_centy_issues(project_name: &str) -> Vec<CentyIssue> {
    info!("running centy list issues for {}", project_name);

    let output = Command::new("pnpm")
        .args([
            "dlx",
            "centy",
            "list",
            "issues",
            "--project",
            project_name,
            "--json",
        ])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            info!("centy issues error: {e}");
            return vec![];
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        info!("centy issues failed: {}", stderr.trim());
        return vec![];
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issues = parse_centy_issues_json(&stdout);

    if issues.is_empty() {
        info!("centy: no issues found for {}", project_name);
    } else {
        info!("centy: {} issue(s) for {}", issues.len(), project_name);
    }

    issues
}
