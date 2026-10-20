use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{default_bg, default_fg};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
    /// Runs while the key is held, for at least `[input] hold_ms`. A short press still runs `action`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold: Option<Action>,
}

/// What a key does when pressed. TOML: `type = "open_url"`, etc.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
        #[cfg_attr(feature = "specta", specta(type = u32))]
        page: usize,
    },
    /// A built-in action from `BUILTIN_NAMES`.
    Builtin {
        name: String,
    },
    /// Claude Code push-to-talk. As a hold: key down at hold start, key up at hold end.
    /// As a tap: one press and release. `key` defaults to `Space`.
    PushToTalk {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
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
    PushToTalk,
}

impl ActionKind {
    pub const ALL: [Self; 8] = [
        Self::None,
        Self::OpenUrl,
        Self::RunCommand,
        Self::OpenApp,
        Self::Hotkey,
        Self::GoToPage,
        Self::Builtin,
        Self::PushToTalk,
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
            Self::PushToTalk => "Push to talk",
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
            hold: None,
        }
    }
}
