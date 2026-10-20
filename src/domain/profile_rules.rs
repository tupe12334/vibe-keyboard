//! Editor rules, shared by every editor (the desktop app and the egui editor).
//! The UI asks for these; it does not repeat them.
use super::keys::is_nav_key;
use super::profile::{Profile, KEY_COUNT};

/// A layout keeps at least this many pages.
pub const MIN_PAGES: usize = 1;

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
    for (index, page) in profile.pages.iter().enumerate() {
        if page.name.trim().is_empty() {
            out.push(format!("page {}: name is empty", index + 1));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::Page;

    #[test]
    fn pages_are_named_by_position_and_never_drop_below_the_minimum() {
        let mut profile = Profile::default();
        assert_eq!(next_page_name(&profile), "Page 1");
        let index = add_page(&mut profile);
        assert_eq!(profile.pages[index].name, "Page 1");
        let second = add_page(&mut profile);
        assert_eq!(profile.pages[second].name, "Page 2");
        assert!(can_delete_page(&profile));
        profile.pages.truncate(MIN_PAGES);
        assert!(!can_delete_page(&profile));
    }

    #[test]
    fn save_refuses_an_empty_page_list_and_blank_names() {
        let empty = Profile {
            pages: vec![],
            ..Profile::default()
        };
        assert!(save_problems(&empty)
            .iter()
            .any(|p| p.contains("at least one page")));
        let blank = Profile {
            pages: vec![Page::new("  ")],
            ..Profile::default()
        };
        assert!(save_problems(&blank)
            .iter()
            .any(|p| p == "page 1: name is empty"));
    }

    #[test]
    fn nav_keys_are_the_reserved_three() {
        assert_eq!(nav_keys(), vec![11, 12, 13]);
    }
}
