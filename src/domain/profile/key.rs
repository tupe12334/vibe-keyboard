use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::color::{default_bg, default_fg};

mod action_kind;
pub use action_kind::ActionKind;

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
    /// A built-in action: the name of a plugin in `crate::plugins::PLUGINS`.
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
