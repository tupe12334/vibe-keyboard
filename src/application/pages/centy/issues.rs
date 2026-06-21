use std::process::Command;
use tracing::info;

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

fn parse_centy_issues_json(json: &str) -> Vec<CentyIssue> {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let arr: Vec<&serde_json::Value> = if let Some(a) = value.as_array() {
        a.iter().collect()
    } else if let Some(a) = value.get("issues").and_then(|v| v.as_array()) {
        a.iter().collect()
    } else {
        return vec![];
    };

    let mut issues: Vec<CentyIssue> = arr
        .into_iter()
        .map(|item| {
            let number = item
                .get("metadata")
                .and_then(|m| m.get("displayNumber"))
                .or_else(|| item.get("displayNumber"))
                .or_else(|| item.get("number"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);

            let title = ["title", "name", "summary"]
                .iter()
                .find_map(|k| item.get(k)?.as_str())
                .unwrap_or("untitled")
                .to_string();

            let status = item
                .get("metadata")
                .and_then(|m| m.get("status"))
                .or_else(|| item.get("status"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let file_path = item
                .get("filePath")
                .or_else(|| item.get("path"))
                .and_then(|v| v.as_str())
                .map(ToString::to_string);

            CentyIssue {
                number,
                title,
                status,
                id,
                file_path,
            }
        })
        .collect();
    issues.retain(|i| i.status != "closed");
    issues.sort_by_key(|i| std::cmp::Reverse(i.number));
    issues
}
