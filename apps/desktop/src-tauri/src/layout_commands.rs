//! Page rules for the editor. The rules live in the core crate; these commands expose them.

use serde::Serialize;
use specta::Type;
use vibe_keyboard::domain::profile::{InputConfig, Profile};
use vibe_keyboard::domain::profile_rules;

/// The editor's rules, read once when the editor opens.
#[derive(Serialize, Type)]
pub struct EditorRules {
    /// Keys the deck reserves for navigation. A profile cannot bind them.
    pub nav_keys: Vec<u8>,
    /// A layout keeps at least this many pages.
    pub min_pages: u32,
    /// The hold time a layout without `[input]` uses, in milliseconds.
    pub default_hold_ms: u32,
}

#[tauri::command]
#[specta::specta]
pub fn editor_rules() -> EditorRules {
    EditorRules {
        nav_keys: profile_rules::nav_keys(),
        min_pages: profile_rules::MIN_PAGES as u32,
        default_hold_ms: InputConfig::default().hold_ms as u32,
    }
}

/// The layout with one more page, named by the core rule.
#[tauri::command]
#[specta::specta]
pub fn add_page(profile: Profile) -> Profile {
    let mut profile = profile;
    profile_rules::add_page(&mut profile);
    profile
}

/// The layout without page `index`. Refused when the layout would have no pages left.
#[tauri::command]
#[specta::specta]
pub fn delete_page(profile: Profile, index: u32) -> Result<Profile, String> {
    let mut profile = profile;
    if !profile_rules::can_delete_page(&profile) {
        return Err("A layout needs at least one page.".into());
    }
    if profile.delete_page(index as usize) {
        Ok(profile)
    } else {
        Err(format!("page {} does not exist", index + 1))
    }
}
