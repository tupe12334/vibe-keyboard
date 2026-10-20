//! Editor rules, shared by every editor (the desktop app and the egui editor).
//! The UI asks for these; it does not repeat them.
use super::keys::is_nav_key;
use super::profile::{Profile, KEY_COUNT};

/// A layout keeps at least this many pages.
pub const MIN_PAGES: usize = 1;

/// Shortest and longest `[input] hold_ms` the editors allow, in milliseconds.
pub const HOLD_MS_MIN: u64 = 50;
pub const HOLD_MS_MAX: u64 = 5000;

/// Name for a new page: "Page N", counting from 1.
pub fn next_page_name(profile: &Profile) -> String {
    format!("Page {}", profile.pages.len() + 1)
}

/// Add a page with the next default name. Returns its index.
pub fn add_page(profile: &mut Profile) -> usize {
    let name = next_page_name(profile);
    profile.add_page(&name)
}

/// Whether the page list may shrink by one.
pub const fn can_delete_page(profile: &Profile) -> bool {
    profile.pages.len() > MIN_PAGES
}

/// Keys the deck reserves for navigation. A profile cannot bind them.
pub fn nav_keys() -> Vec<u8> {
    (1..=KEY_COUNT).filter(|&key| is_nav_key(key)).collect()
}

/// Problems that block a save: the layout problems, plus the page-list rules.
pub fn save_problems(profile: &Profile) -> Vec<String> {
    let mut out = profile.problems();
    if profile.pages.len() < MIN_PAGES {
        out.push("the layout needs at least one page".into());
    }
    if !(HOLD_MS_MIN..=HOLD_MS_MAX).contains(&profile.input.hold_ms) {
        out.push(format!(
            "hold time {} ms must be {HOLD_MS_MIN} to {HOLD_MS_MAX}",
            profile.input.hold_ms
        ));
    }
    for (index, page) in profile.pages.iter().enumerate() {
        if page.name.trim().is_empty() {
            out.push(format!("page {}: name is empty", index + 1));
        }
    }
    out
}

#[cfg(test)]
#[path = "profile_rules_tests.rs"]
mod tests;
