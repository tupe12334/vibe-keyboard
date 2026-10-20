#[cfg(test)]
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod display;
mod input_config;
mod layout;
mod legacy;
mod page;
pub use display::{inner_rect, is_default, DisplaySettings, KeyInset, PixelRect, SideInset};
pub use input_config::InputConfig;
pub use page::Page;
mod key;
pub use key::{Action, ActionKind, KeyConfig};
mod side_config;
mod validate;
pub use side_config::{SideConfig, Widget};

/// Largest physical key number. Keys 1–15 are usable; 11–13 are navigation.
pub const KEY_COUNT: u8 = 15;

/// Names accepted by `Action::Builtin`. Each one maps to hard-coded code in the app.
pub const BUILTIN_NAMES: [&str; 9] = [
    "centy_projects",
    "terminal",
    "claude",
    "centy_web",
    "numpad",
    "log_file",
    "vscode_config",
    "claude_sessions",
    "moadim",
];

const DEFAULT_BG: &str = "#141414";
const DEFAULT_FG: &str = "#E6E6E6";

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
}

/// A prompt a session action types into a Claude Code session, followed by Enter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Prompt {
    pub label: String,
    pub text: String,
}

/// Longest prompt text the profile accepts.
pub const PROMPT_MAX_CHARS: usize = 2000;

/// Longest prompt label the profile accepts. Labels print on a key image.
pub const PROMPT_LABEL_MAX_CHARS: usize = 40;

fn default_bg() -> String {
    DEFAULT_BG.into()
}

fn default_fg() -> String {
    DEFAULT_FG.into()
}

/// True for macOS bundle ids such as `com.apple.Terminal`: dotted, no spaces.
/// Names ending in `.app` are app names (`Google Chrome.app`), not bundle ids.
pub fn is_bundle_id(text: &str) -> bool {
    text.contains('.') && !text.contains(' ') && !text.ends_with(".app")
}

impl Action {
    pub const fn kind(&self) -> ActionKind {
        match self {
            Self::None => ActionKind::None,
            Self::OpenUrl { .. } => ActionKind::OpenUrl,
            Self::RunCommand { .. } => ActionKind::RunCommand,
            Self::OpenApp { .. } => ActionKind::OpenApp,
            Self::Hotkey { .. } => ActionKind::Hotkey,
            Self::GoToPage { .. } => ActionKind::GoToPage,
            Self::Builtin { .. } => ActionKind::Builtin,
            Self::PushToTalk { .. } => ActionKind::PushToTalk,
        }
    }

    /// A fresh action of `kind` with empty or first-choice fields.
    pub fn default_of(kind: ActionKind) -> Self {
        match kind {
            ActionKind::None => Self::None,
            ActionKind::OpenUrl => Self::OpenUrl { url: String::new() },
            ActionKind::RunCommand => Self::RunCommand {
                command: String::new(),
            },
            ActionKind::OpenApp => Self::OpenApp {
                name: String::new(),
            },
            ActionKind::Hotkey => Self::Hotkey {
                keys: String::new(),
            },
            ActionKind::GoToPage => Self::GoToPage { page: 0 },
            ActionKind::PushToTalk => Self::PushToTalk { key: None },
            ActionKind::Builtin => Self::Builtin {
                name: BUILTIN_NAMES[0].into(),
            },
        }
    }
}

/// Parse `#RRGGBB` into bytes. Returns `None` for any other text.
pub fn parse_color(text: &str) -> Option<[u8; 3]> {
    let hex = text.strip_prefix('#')?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    Some([(value >> 16) as u8, (value >> 8) as u8, value as u8])
}

/// Format bytes as `#RRGGBB`.
pub fn format_color(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "profile_input_tests.rs"]
mod input_tests;
