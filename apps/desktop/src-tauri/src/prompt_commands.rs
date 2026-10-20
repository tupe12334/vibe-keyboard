//! Session prompt row edits for the editor. The row rules live in the core crate.
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::domain::prompt_rules;

/// The layout with one more prompt row, with the core rule's default text.
#[tauri::command]
#[specta::specta]
pub fn add_prompt(profile: Profile) -> Profile {
    let mut profile = profile;
    prompt_rules::add_prompt(&mut profile.prompts);
    profile
}

/// The layout with prompt row `index` swapped with its neighbour above or below.
#[tauri::command]
#[specta::specta]
pub fn move_prompt(profile: Profile, index: u32, down: bool) -> Profile {
    let mut profile = profile;
    prompt_rules::move_prompt(&mut profile.prompts, index as usize, down);
    profile
}

/// The layout without prompt row `index`.
#[tauri::command]
#[specta::specta]
pub fn delete_prompt(profile: Profile, index: u32) -> Profile {
    let mut profile = profile;
    prompt_rules::delete_prompt(&mut profile.prompts, index as usize);
    profile
}
