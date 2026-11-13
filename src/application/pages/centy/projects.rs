use std::time::Duration;

use tracing::info;

use crate::domain::actions::CentyProject;
use crate::infrastructure::process::probe;

// ponytail: pnpm dlx may download centy on first run. A hung call is killed so the loading tile clears.
const CENTY_TIMEOUT: Duration = Duration::from_secs(10);

pub fn fetch_centy_projects() -> Vec<CentyProject> {
    info!("running centy list projects");

    let Some(stdout) = probe(
        "pnpm",
        &["dlx", "centy", "list", "projects", "--json"],
        CENTY_TIMEOUT,
    ) else {
        info!("centy: no output, failed, or timed out");
        return vec![];
    };
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
                .map(ToString::to_string);

            let url = format!("https://app.centy.io/{org}/{name}");

            CentyProject {
                name,
                org,
                path,
                url,
            }
        })
        .collect()
}
