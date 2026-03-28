use std::process::Command;
use tracing::info;

use crate::domain::actions::{CentyIssue, CentyProject};

pub fn fetch_centy_projects() -> Vec<CentyProject> {
    info!("running centy list projects");

    let output = Command::new("pnpm")
        .args(["dlx", "centy", "list", "projects", "--json"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            info!("centy error: {e}");
            return vec![];
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        info!("centy failed: {}", stderr.trim());
        return vec![];
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let projects = parse_centy_json(&stdout);

    if projects.is_empty() {
        info!("centy: no projects found");
    } else {
        info!("centy: {} project(s)", projects.len());
    }

    projects
}

fn parse_centy_json(json: &str) -> Vec<CentyProject> {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    // Accept either a top-level array or an object with a "projects" key
    let arr: Vec<&serde_json::Value> = if let Some(a) = value.as_array() {
        a.iter().collect()
    } else if let Some(a) = value.get("projects").and_then(|v| v.as_array()) {
        a.iter().collect()
    } else {
        return vec![];
    };

    arr.into_iter()
        .map(|item| {
            let name = ["name", "slug", "title"]
                .iter()
                .find_map(|k| item.get(k)?.as_str())
                .unwrap_or("unknown")
                .to_string();

            let org = [
                "organizationSlug",
                "organizationName",
                "org",
                "organization",
                "owner",
                "workspace",
            ]
            .iter()
            .find_map(|k| item.get(k)?.as_str())
            .unwrap_or("unknown")
            .to_string();

            let path = ["path", "directory", "dir", "root", "localPath"]
                .iter()
                .find_map(|k| item.get(k)?.as_str())
                .map(|s| s.to_string());

            let url = format!("https://app.centy.io/{}/{}", org, name);

            CentyProject {
                name,
                org,
                path,
                url,
            }
        })
        .collect()
}

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

    arr.into_iter()
        .map(|item| {
            let number = item
                .get("displayNumber")
                .or_else(|| item.get("number"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let title = ["title", "name", "summary"]
                .iter()
                .find_map(|k| item.get(k)?.as_str())
                .unwrap_or("untitled")
                .to_string();

            let status = item
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            CentyIssue {
                number,
                title,
                status,
            }
        })
        .collect()
}
