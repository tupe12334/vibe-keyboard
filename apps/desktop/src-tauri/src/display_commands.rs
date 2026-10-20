//! Display insets: the built-in values the editor starts from when a layout leaves them out.

use serde::Serialize;
use specta::Type;
use vibe_keyboard::domain::profile::{DisplaySettings, KeyInset, SideInset};

#[derive(Serialize, Type)]
pub struct DisplayDefaults {
    pub key_inset: KeyInset,
    pub side_inset: SideInset,
}

/// The built-in insets. The layout file omits a value that equals its default.
#[tauri::command]
#[specta::specta]
pub fn display_defaults() -> DisplayDefaults {
    let defaults = DisplaySettings::default();
    DisplayDefaults {
        key_inset: defaults.key_inset,
        side_inset: defaults.side_inset,
    }
}
