use crate::domain::actions::CentyIssue;

pub(super) fn parse_centy_issues_json(json: &str) -> Vec<CentyIssue> {
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
