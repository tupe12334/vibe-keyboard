#[cfg(test)]
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod bar_style;
mod builtins;
mod claude_sessions_config;
mod color;
mod display;
mod input_config;
mod layout;
mod legacy;
mod page;
mod prompt;
pub use bar_style::*;
pub use builtins::BUILTIN_NAMES;
pub use claude_sessions_config::ClaudeSessionsConfig;
#[cfg(test)]
use color::DEFAULT_BG;
pub use color::{format_color, parse_color};
pub use display::{inner_rect, is_default, DisplaySettings, KeyInset, PixelRect, SideInset};
pub use input_config::InputConfig;
pub use page::Page;
pub use prompt::{Prompt, PROMPT_LABEL_MAX_CHARS, PROMPT_MAX_CHARS};
mod key;
pub use key::{Action, ActionKind, KeyConfig};
mod side_config;
mod validate;
pub use side_config::{SideConfig, Widget};

/// Largest physical key number. Keys 1–15 are usable; 11–13 are navigation.
pub const KEY_COUNT: u8 = 15;

/// The whole user layout: an ordered list of main pages, and the prompts a session action sends.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Profile {
    #[serde(default)]
    pub pages: Vec<Page>,
    /// Text typed into a Claude Code session by its action page. Empty means the built-in pair.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prompts: Vec<Prompt>,
    /// Device-wide display settings: the band the bezel hides, in pixels.
    #[serde(default, skip_serializing_if = "is_default")]
    pub display: DisplaySettings,
    /// Timing of key holds. Absent means the defaults, so older files still load.
    #[serde(default, skip_serializing_if = "InputConfig::is_default")]
    pub input: InputConfig,
    /// Options for the Claude session screens. Absent means the defaults.
    #[serde(default, skip_serializing_if = "ClaudeSessionsConfig::is_default")]
    pub claude_sessions: ClaudeSessionsConfig,
}

/// True for macOS bundle ids such as `com.apple.Terminal`: dotted, no spaces.
/// Names ending in `.app` are app names (`Google Chrome.app`), not bundle ids.
/// The suffix is matched case-insensitively, because macOS file systems are
/// case-insensitive by default and an app can be named `Foo.App`.
pub fn is_bundle_id(text: &str) -> bool {
    text.contains('.') && !text.contains(' ') && !text.to_ascii_lowercase().ends_with(".app")
}

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "profile_input_tests.rs"]
mod input_tests;

#[cfg(test)]
#[path = "profile_claude_sessions_tests.rs"]
mod claude_sessions_tests;
