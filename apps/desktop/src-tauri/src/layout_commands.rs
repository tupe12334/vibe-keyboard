//! Page rules for the editor. The rules live in the core crate; these commands expose them.

use serde::Serialize;
use specta::Type;
use vibe_keyboard::domain::profile::{
    InputConfig, Profile, PROMPT_LABEL_MAX_CHARS, PROMPT_MAX_CHARS,
};
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
    /// Shortest and longest hold time the editor accepts, in milliseconds.
    pub hold_ms_min: u32,
    pub hold_ms_max: u32,
    /// Longest prompt label, in characters. Labels print on a key image.
    pub prompt_label_max: u32,
    /// Longest prompt text, in characters.
    pub prompt_text_max: u32,
}

#[tauri::command]
#[specta::specta]
pub fn editor_rules() -> EditorRules {
    EditorRules {
        nav_keys: profile_rules::nav_keys(),
        min_pages: profile_rules::MIN_PAGES as u32,
        default_hold_ms: InputConfig::default().hold_ms as u32,
        hold_ms_min: profile_rules::HOLD_MS_MIN as u32,
        hold_ms_max: profile_rules::HOLD_MS_MAX as u32,
        prompt_label_max: PROMPT_LABEL_MAX_CHARS as u32,
        prompt_text_max: PROMPT_MAX_CHARS as u32,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_rules_expose_the_prompt_limits_from_the_core_crate() {
        let rules = editor_rules();
        assert_eq!(rules.prompt_label_max as usize, PROMPT_LABEL_MAX_CHARS);
        assert_eq!(rules.prompt_text_max as usize, PROMPT_MAX_CHARS);
    }
}
