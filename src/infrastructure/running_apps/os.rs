//! The `lsappinfo` call for `running_apps`. Excluded from the coverage gate in `.cargo-husky/hooks/pre-push`.

use std::process::Command;

/// Running apps as (name, bundle id), from `lsappinfo list`. Empty if it fails.
pub fn running_apps() -> Vec<(String, String)> {
    Command::new("lsappinfo")
        .arg("list")
        .output()
        .map(|out| super::parse_lsappinfo(&String::from_utf8_lossy(&out.stdout)))
        .unwrap_or_default()
}
