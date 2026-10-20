//! Extends `PATH` at process start. A login item (launchd) gets only `/usr/bin:/bin:/usr/sbin:/sbin`,
//! so tools in Homebrew, `~/.local/bin` or Nix are not found.
//!
//! This file holds the pure merge. The env and filesystem reads live in `login_path/os.rs`.

use std::path::Path;

// The `#[path]` keeps the test file next to this one, as in `app_icon.rs`.
#[path = "login_path/os.rs"]
mod os;

pub use os::apply_login_path;

/// Tool directories added when they exist. Order is the order they are appended in.
const EXTRA_DIRS: [&str; 8] = [
    "$HOME/.local/bin",
    "/opt/homebrew/bin",
    "/opt/homebrew/sbin",
    "/usr/local/bin",
    "$HOME/.nix-profile/bin",
    "/nix/var/nix/profiles/default/bin",
    "$HOME/.volta/bin",
    "$HOME/.cargo/bin",
];

/// Merges the extra tool directories into `current` (a `PATH` string).
///
/// Current entries keep their order and come first. An extra directory is appended only when
/// `exists` says it is there and it is not already listed. Empty entries are dropped. A `$HOME`
/// entry is skipped when `home` is empty, so no relative path is added.
pub fn merge_path(current: &str, home: &str, exists: impl Fn(&Path) -> bool) -> String {
    let mut merged: Vec<String> = Vec::new();
    for entry in current.split(':').filter(|entry| !entry.is_empty()) {
        if !merged.iter().any(|kept| kept == entry) {
            merged.push(entry.to_owned());
        }
    }
    for template in EXTRA_DIRS {
        let Some(dir) = expand_home(template, home) else {
            continue;
        };
        if !merged.contains(&dir) && exists(Path::new(&dir)) {
            merged.push(dir);
        }
    }
    merged.join(":")
}

/// Replaces a leading `$HOME` with `home`. Returns `None` when `home` is empty and the template needs it.
fn expand_home(template: &str, home: &str) -> Option<String> {
    match template.strip_prefix("$HOME") {
        None => Some(template.to_owned()),
        Some(_) if home.is_empty() => None,
        Some(rest) => Some(format!("{home}{rest}")),
    }
}

#[cfg(test)]
#[path = "login_path_tests.rs"]
mod tests;
