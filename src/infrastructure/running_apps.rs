//! Apps running now, as (name, bundle id). The app-icon pickers in both editors list them.

#[path = "running_apps/os.rs"]
mod os;

pub use os::running_apps;

/// Parse `lsappinfo list`. Each app starts with `N) "Name"` and has a `bundleID="..."` line.
pub fn parse_lsappinfo(text: &str) -> Vec<(String, String)> {
    let mut apps = Vec::new();
    let mut name: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some((_, rest)) = trimmed
            .split_once(") \"")
            .filter(|(number, _)| number.chars().all(|c| c.is_ascii_digit()))
        {
            name = rest.split('"').next().map(str::to_string);
        } else if let Some(rest) = trimmed.strip_prefix("bundleID=\"") {
            if let (Some(app), Some(id)) = (name.take(), rest.split('"').next()) {
                if id != "[ NULL ]" && !id.is_empty() {
                    apps.push((app, id.to_string()));
                }
            }
        }
    }
    apps
}

#[cfg(test)]
#[path = "running_apps_tests.rs"]
mod tests;
