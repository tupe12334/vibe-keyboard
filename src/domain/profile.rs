use std::collections::BTreeMap;
#[cfg(test)]
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

mod display;
pub use display::{inner_rect, is_default, DisplaySettings, KeyInset, PixelRect, SideInset};
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

/// The `[input]` table of `profile.toml`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputConfig {
    /// Milliseconds a key must stay down before its `hold` action starts.
    #[serde(default = "default_hold_ms")]
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub hold_ms: u64,
}

const DEFAULT_HOLD_MS: u64 = 400;

const fn default_hold_ms() -> u64 {
    DEFAULT_HOLD_MS
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            hold_ms: DEFAULT_HOLD_MS,
        }
    }
}

impl InputConfig {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Page {
    pub name: String,
    #[serde(default, with = "key_map")]
    #[cfg_attr(
        feature = "specta",
        specta(type = std::collections::BTreeMap<String, KeyConfig>)
    )]
    pub keys: BTreeMap<u8, KeyConfig>,
    /// Bundle ids (for example `com.google.Chrome`). The page shows when one is in front.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apps: Vec<String>,
    /// Content for the side screen next to keys 5, 10 and 15. Absent means no side content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<SideConfig>,
}

/// TOML tables need string keys, so key numbers are written as `"1"`, `"14"`, etc.
/// Output order is the string order of the keys (`"1", "10", ...`). It is still deterministic.
mod key_map {
    use std::collections::BTreeMap;

    use serde::de::Error as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::KeyConfig;

    pub fn serialize<S: Serializer>(
        map: &BTreeMap<u8, KeyConfig>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let text: BTreeMap<String, &KeyConfig> =
            map.iter().map(|(k, v)| (k.to_string(), v)).collect();
        text.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<u8, KeyConfig>, D::Error> {
        let text: BTreeMap<String, KeyConfig> = BTreeMap::deserialize(deserializer)?;
        text.into_iter()
            .map(|(k, v)| {
                k.parse::<u8>()
                    .map(|key| (key, v))
                    .map_err(D::Error::custom)
            })
            .collect()
    }
}

fn default_bg() -> String {
    DEFAULT_BG.into()
}

fn default_fg() -> String {
    DEFAULT_FG.into()
}

impl Page {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            keys: BTreeMap::new(),
            apps: Vec::new(),
            side: None,
        }
    }
}

/// True for macOS bundle ids such as `com.apple.Terminal`: dotted, no spaces.
/// Names ending in `.app` are app names (`Google Chrome.app`), not bundle ids.
pub fn is_bundle_id(text: &str) -> bool {
    text.contains('.') && !text.contains(' ') && !text.ends_with(".app")
}

impl Profile {
    /// How long a key must stay down to count as a hold.
    pub const fn hold_threshold(&self) -> Duration {
        Duration::from_millis(self.input.hold_ms)
    }

    /// The layout that matches the hard-coded main pages before profiles existed.
    pub fn default_from_legacy() -> Self {
        let builtin = |label: &str, name: &str| KeyConfig {
            label: label.into(),
            action: Action::Builtin { name: name.into() },
            ..KeyConfig::default()
        };
        Self {
            prompts: Vec::new(),
            input: InputConfig::default(),
            pages: vec![
                Page {
                    name: "Main".into(),
                    apps: Vec::new(),
                    side: None,
                    keys: BTreeMap::from([
                        (1, builtin("List Projects", "centy_projects")),
                        (2, builtin("Open Terminal", "terminal")),
                        (3, builtin("Open Claude", "claude")),
                        (4, builtin("Open Centy", "centy_web")),
                    ]),
                },
                Page {
                    name: "Tools".into(),
                    apps: Vec::new(),
                    side: None,
                    keys: BTreeMap::from([
                        (1, builtin("Input Number", "numpad")),
                        (14, builtin("Open Log", "log_file")),
                        (15, builtin("Open Config", "vscode_config")),
                    ]),
                },
            ],
            display: DisplaySettings::default(),
        }
    }

    /// First page whose `apps` lists `bundle_id`.
    pub fn page_for_app(&self, bundle_id: &str) -> Option<usize> {
        self.pages
            .iter()
            .position(|page| page.apps.iter().any(|app| app == bundle_id))
    }

    pub fn key_at(&self, page: usize, key: u8) -> Option<&KeyConfig> {
        self.pages.get(page)?.keys.get(&key)
    }

    pub fn action_at(&self, page: usize, key: u8) -> Option<&Action> {
        self.key_at(page, key).map(|k| &k.action)
    }

    /// Store `config` at `key` on `page`. Does nothing if the page does not exist.
    pub fn set_key(&mut self, page: usize, key: u8, config: KeyConfig) {
        if let Some(p) = self.pages.get_mut(page) {
            p.keys.insert(key, config);
        }
    }

    /// Set or clear the side screen of `page`. Does nothing if the page does not exist.
    pub fn set_side(&mut self, page: usize, side: Option<SideConfig>) {
        if let Some(p) = self.pages.get_mut(page) {
            p.side = side;
        }
    }

    pub fn clear_key(&mut self, page: usize, key: u8) {
        if let Some(p) = self.pages.get_mut(page) {
            p.keys.remove(&key);
        }
    }

    /// Append an empty page and return its index.
    pub fn add_page(&mut self, name: &str) -> usize {
        self.pages.push(Page::new(name));
        self.pages.len() - 1
    }

    /// Remove a page. Refuses to remove the last one, so the device always has a page.
    pub fn delete_page(&mut self, index: usize) -> bool {
        if self.pages.len() <= 1 || index >= self.pages.len() {
            return false;
        }
        self.pages.remove(index);
        true
    }

    /// Move a page from one index to another. Returns `false` if an index is out of range.
    pub fn move_page(&mut self, from: usize, to: usize) -> bool {
        if from >= self.pages.len() || to >= self.pages.len() {
            return false;
        }
        let page = self.pages.remove(from);
        self.pages.insert(to, page);
        true
    }

    /// Add an empty page when the file has none, because the device needs one.
    pub fn ensure_one_page(&mut self) {
        if self.pages.is_empty() {
            self.pages.push(Page::new("Page 1"));
        }
    }

    /// The prompts a session action can send. Empty when the profile has none.
    pub fn session_prompts(&self) -> Vec<Prompt> {
        self.prompts.clone()
    }
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
