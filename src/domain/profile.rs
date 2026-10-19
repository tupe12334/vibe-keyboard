use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::hotkey::parse_hotkey;
use super::keys::is_nav_key;

/// Largest physical key number. Keys 1–15 are usable; 11–13 are navigation.
pub const KEY_COUNT: u8 = 15;

/// Names accepted by `Action::Builtin`. Each one maps to hard-coded code in the app.
pub const BUILTIN_NAMES: [&str; 7] = [
    "centy_projects",
    "terminal",
    "claude",
    "centy_web",
    "numpad",
    "log_file",
    "vscode_config",
];

const DEFAULT_BG: &str = "#141414";
const DEFAULT_FG: &str = "#E6E6E6";

/// The whole user layout: an ordered list of main pages.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub name: String,
    #[serde(default, with = "key_map")]
    pub keys: BTreeMap<u8, KeyConfig>,
    /// Bundle ids (for example `com.google.Chrome`). The page shows when one is in front.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apps: Vec<String>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyConfig {
    #[serde(default)]
    pub label: String,
    #[serde(default = "default_bg")]
    pub bg: String,
    #[serde(default = "default_fg")]
    pub fg: String,
    /// Shown instead of the label when set. Also drawn for `Builtin` keys without it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<PathBuf>,
    #[serde(default)]
    pub action: Action,
}

/// What a key does when pressed. TOML: `type = "open_url"`, etc.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    #[default]
    None,
    OpenUrl {
        url: String,
    },
    /// Run through `sh -c`, detached.
    RunCommand {
        command: String,
    },
    /// Run `open -a <name>`.
    OpenApp {
        name: String,
    },
    /// Send a shortcut such as `cmd+shift+4` through System Events.
    Hotkey {
        keys: String,
    },
    /// Jump to a main page by its 0-based index.
    GoToPage {
        page: usize,
    },
    /// A built-in action from `BUILTIN_NAMES`.
    Builtin {
        name: String,
    },
}

/// The kind of an `Action`, without its fields. Used by the editor's type picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    None,
    OpenUrl,
    RunCommand,
    OpenApp,
    Hotkey,
    GoToPage,
    Builtin,
}

impl ActionKind {
    pub const ALL: [Self; 7] = [
        Self::None,
        Self::OpenUrl,
        Self::RunCommand,
        Self::OpenApp,
        Self::Hotkey,
        Self::GoToPage,
        Self::Builtin,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::OpenUrl => "Open URL",
            Self::RunCommand => "Run command",
            Self::OpenApp => "Open app",
            Self::Hotkey => "Hotkey",
            Self::GoToPage => "Go to page",
            Self::Builtin => "Built-in",
        }
    }
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            bg: default_bg(),
            fg: default_fg(),
            image: None,
            action: Action::None,
        }
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
        }
    }
}

/// True for macOS bundle ids such as `com.apple.Terminal`: dotted, no spaces.
/// Names ending in `.app` are app names (`Google Chrome.app`), not bundle ids.
pub fn is_bundle_id(text: &str) -> bool {
    text.contains('.') && !text.contains(' ') && !text.ends_with(".app")
}

impl Profile {
    /// The layout that matches the hard-coded main pages before profiles existed.
    pub fn default_from_legacy() -> Self {
        let builtin = |label: &str, name: &str| KeyConfig {
            label: label.into(),
            action: Action::Builtin { name: name.into() },
            ..KeyConfig::default()
        };
        Self {
            pages: vec![
                Page {
                    name: "Main".into(),
                    apps: Vec::new(),
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
                    keys: BTreeMap::from([
                        (1, builtin("Input Number", "numpad")),
                        (14, builtin("Open Log", "log_file")),
                        (15, builtin("Open Config", "vscode_config")),
                    ]),
                },
            ],
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

    /// Human-readable problems in the layout. Empty when the profile is valid.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (index, page) in self.pages.iter().enumerate() {
            for (key, config) in &page.keys {
                let place = format!("page {} '{}' key {key}", index + 1, page.name);
                if *key == 0 || *key > KEY_COUNT || is_nav_key(*key) {
                    out.push(format!("{place}: key is reserved or out of range"));
                }
                if config
                    .image
                    .as_deref()
                    .is_some_and(|p| p.as_os_str().is_empty())
                {
                    out.push(format!("{place}: image is empty"));
                }
                if parse_color(&config.bg).is_none() {
                    out.push(format!("{place}: bg must be #RRGGBB"));
                }
                if parse_color(&config.fg).is_none() {
                    out.push(format!("{place}: fg must be #RRGGBB"));
                }
                if let Some(problem) = config.action.problem(self.pages.len()) {
                    out.push(format!("{place}: {problem}"));
                }
            }
        }
        out
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
            ActionKind::Builtin => Self::Builtin {
                name: BUILTIN_NAMES[0].into(),
            },
        }
    }

    /// Reason this action cannot run, if any. `page_count` is the number of pages in the profile.
    pub fn problem(&self, page_count: usize) -> Option<String> {
        match self {
            Self::None => None,
            Self::OpenUrl { url } => url.trim().is_empty().then(|| "url is empty".into()),
            Self::RunCommand { command } => {
                command.trim().is_empty().then(|| "command is empty".into())
            }
            Self::OpenApp { name } => name.trim().is_empty().then(|| "app name is empty".into()),
            Self::Hotkey { keys } => parse_hotkey(keys).err(),
            Self::GoToPage { page } => {
                (*page >= page_count).then(|| format!("go-to page {page} does not exist"))
            }
            Self::Builtin { name } => (!BUILTIN_NAMES.contains(&name.as_str()))
                .then(|| format!("unknown built-in '{name}'")),
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
mod tests {
    use super::*;

    #[test]
    fn toml_round_trip_keeps_every_action() {
        let mut profile = Profile::default_from_legacy();
        let page = profile.add_page("Extra");
        let actions = [
            Action::OpenUrl {
                url: "https://example.com".into(),
            },
            Action::RunCommand {
                command: "echo hi".into(),
            },
            Action::OpenApp {
                name: "Notes".into(),
            },
            Action::Hotkey {
                keys: "cmd+shift+4".into(),
            },
            Action::GoToPage { page: 0 },
        ];
        for (i, action) in actions.into_iter().enumerate() {
            let cfg = KeyConfig {
                label: "x".into(),
                action,
                image: Some(PathBuf::from("/tmp/a.png")),
                ..KeyConfig::default()
            };
            profile.set_key(page, u8::try_from(i + 1).unwrap_or(1), cfg);
        }
        let text = toml::to_string_pretty(&profile).unwrap_or_default();
        let back: Profile = toml::from_str(&text).unwrap_or_default();
        assert_eq!(back, profile, "text was:\n{text}");
    }

    #[test]
    fn missing_fields_take_defaults() {
        let text = "[[pages]]\nname = \"A\"\n\n[pages.keys.2]\nlabel = \"Hi\"\n";
        let profile: Profile = toml::from_str(text).unwrap_or_default();
        let key = profile.key_at(0, 2);
        assert_eq!(key.map(|k| k.bg.as_str()), Some(DEFAULT_BG));
        assert_eq!(key.map(|k| &k.action), Some(&Action::None));
    }

    #[test]
    fn action_parses_each_variant_from_toml() {
        let cases = [
            ("type = \"none\"", Action::None),
            (
                "type = \"open_url\"\nurl = \"u\"",
                Action::OpenUrl { url: "u".into() },
            ),
            (
                "type = \"run_command\"\ncommand = \"c\"",
                Action::RunCommand {
                    command: "c".into(),
                },
            ),
            (
                "type = \"open_app\"\nname = \"n\"",
                Action::OpenApp { name: "n".into() },
            ),
            (
                "type = \"hotkey\"\nkeys = \"cmd+a\"",
                Action::Hotkey {
                    keys: "cmd+a".into(),
                },
            ),
            (
                "type = \"go_to_page\"\npage = 1",
                Action::GoToPage { page: 1 },
            ),
            (
                "type = \"builtin\"\nname = \"terminal\"",
                Action::Builtin {
                    name: "terminal".into(),
                },
            ),
        ];
        for (text, expected) in cases {
            let parsed: Action = toml::from_str(text).unwrap_or_default();
            assert_eq!(parsed, expected, "input: {text}");
        }
    }

    #[test]
    fn legacy_profile_is_valid() {
        assert!(Profile::default_from_legacy().problems().is_empty());
    }

    #[test]
    fn problems_flag_reserved_keys_colors_and_actions() {
        let mut profile = Profile::default_from_legacy();
        profile.set_key(
            0,
            12,
            KeyConfig {
                bg: "red".into(),
                action: Action::GoToPage { page: 9 },
                ..KeyConfig::default()
            },
        );
        assert_eq!(profile.problems().len(), 3);
    }

    #[test]
    fn color_parsing_round_trips() {
        assert_eq!(parse_color("#1A2B3C"), Some([0x1A, 0x2B, 0x3C]));
        assert_eq!(parse_color("1A2B3C"), None);
        assert_eq!(parse_color("#+A2B3C"), None);
        assert_eq!(format_color([0x1A, 0x2B, 0x3C]), "#1A2B3C");
    }

    #[test]
    fn delete_keeps_at_least_one_page_and_move_reorders() {
        let mut profile = Profile::default_from_legacy();
        assert!(profile.move_page(0, 1));
        assert_eq!(profile.pages[0].name, "Tools");
        assert!(profile.delete_page(0));
        assert!(!profile.delete_page(0));
    }

    #[test]
    fn edit_methods_update_pages_and_keys() {
        let mut profile = Profile::default();
        profile.ensure_one_page();
        assert_eq!(profile.pages.len(), 1);
        let second = profile.add_page("Two");
        assert_eq!(second, 1);
        profile.set_key(
            1,
            3,
            KeyConfig {
                label: "x".into(),
                ..KeyConfig::default()
            },
        );
        profile.set_key(9, 3, KeyConfig::default());
        assert_eq!(profile.key_at(1, 3).map(|k| k.label.as_str()), Some("x"));
        assert!(profile.action_at(1, 3).is_some());
        assert!(profile.key_at(9, 3).is_none());
        profile.clear_key(1, 3);
        profile.clear_key(9, 3);
        assert!(profile.key_at(1, 3).is_none());
        assert!(!profile.move_page(0, 5));
        assert!(!profile.move_page(5, 0));
        assert!(!profile.delete_page(7));
    }

    #[test]
    fn action_problems_cover_each_variant() {
        assert_eq!(Action::None.problem(1), None);
        assert!(Action::OpenUrl { url: " ".into() }.problem(1).is_some());
        assert_eq!(Action::OpenUrl { url: "u".into() }.problem(1), None);
        assert!(Action::RunCommand {
            command: String::new()
        }
        .problem(1)
        .is_some());
        assert!(Action::OpenApp {
            name: String::new()
        }
        .problem(1)
        .is_some());
        assert_eq!(
            Action::OpenApp {
                name: "Notes".into()
            }
            .problem(1),
            None
        );
        assert!(Action::Hotkey {
            keys: "hyper+a".into()
        }
        .problem(1)
        .is_some());
        assert!(Action::GoToPage { page: 3 }.problem(2).is_some());
        assert_eq!(Action::GoToPage { page: 1 }.problem(2), None);
        assert!(Action::Builtin {
            name: "nope".into()
        }
        .problem(1)
        .is_some());
    }

    #[test]
    fn action_kinds_match_defaults_and_labels() {
        for kind in ActionKind::ALL {
            assert_eq!(Action::default_of(kind).kind(), kind);
            assert!(!kind.label().is_empty());
        }
    }

    #[test]
    fn invalid_foreground_color_is_reported() {
        let mut profile = Profile::default_from_legacy();
        profile.set_key(
            0,
            2,
            KeyConfig {
                fg: "white".into(),
                ..KeyConfig::default()
            },
        );
        assert_eq!(profile.problems().len(), 1);
    }

    #[test]
    fn empty_image_path_is_reported() {
        let mut profile = Profile::default_from_legacy();
        profile.set_key(
            0,
            2,
            KeyConfig {
                image: Some(PathBuf::new()),
                ..KeyConfig::default()
            },
        );
        let problems = profile.problems();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("image is empty"));
    }

    #[test]
    fn non_numeric_key_is_rejected() {
        let text = "[[pages]]\nname = \"A\"\n\n[pages.keys.x]\nlabel = \"Hi\"\n";
        assert!(toml::from_str::<Profile>(text).is_err());
    }

    #[test]
    fn page_for_app_picks_first_match() {
        let mut profile = Profile::default_from_legacy();
        profile.pages[1].apps = vec!["com.google.Chrome".into()];
        profile.pages[0].apps = vec!["com.google.Chrome".into()];
        assert_eq!(profile.page_for_app("com.google.Chrome"), Some(0));
        assert_eq!(profile.page_for_app("com.apple.Notes"), None);
    }

    #[test]
    fn apps_field_is_omitted_when_empty() {
        let text = toml::to_string_pretty(&Profile::default_from_legacy()).unwrap_or_default();
        assert!(!text.contains("apps"));
        let mut profile = Profile::default_from_legacy();
        profile.pages[0].apps = vec!["com.google.Chrome".into()];
        let with = toml::to_string_pretty(&profile).unwrap_or_default();
        assert!(with.contains("apps = [\"com.google.Chrome\"]"));
        let back: Profile = toml::from_str(&with).unwrap_or_default();
        assert_eq!(back, profile);
    }

    #[test]
    fn bundle_id_detection() {
        assert!(is_bundle_id("com.apple.Terminal"));
        assert!(!is_bundle_id("Terminal"));
        assert!(!is_bundle_id("My App.app"));
        assert!(!is_bundle_id("Chrome.app"));
    }
}
