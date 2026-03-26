use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level profile loaded from a TOML file.
///
/// Example config.toml:
/// ```toml
/// brightness = 60
/// boot_logo  = "logo.jpg"
///
/// [[buttons]]
/// index = 0
/// image = "images/terminal.jpg"
/// [buttons.action]
/// type    = "launch"
/// command = "kitty"
///
/// [[buttons]]
/// index = 2
/// [buttons.action]
/// type    = "keypress"
/// keys    = "ctrl+shift+t"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Display brightness 0–100 (default: 75)
    #[serde(default = "default_brightness")]
    pub brightness: u8,

    /// Optional path to a boot logo image written to the device on startup
    pub boot_logo: Option<PathBuf>,

    /// Per-button configuration
    #[serde(default)]
    pub buttons: Vec<ButtonConfig>,
}

fn default_brightness() -> u8 {
    75
}

impl Default for Config {
    fn default() -> Self {
        Config {
            brightness: default_brightness(),
            boot_logo: None,
            buttons: Vec::new(),
        }
    }
}

/// Configuration for a single button.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ButtonConfig {
    /// 0-based button index on the device
    pub index: u8,

    /// Optional image shown on this button (JPEG, PNG, BMP, …)
    pub image: Option<PathBuf>,

    /// Action to run when the button is pressed
    pub action: Option<Action>,
}

/// What happens when a button is pressed.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Spawn an application.
    /// ```toml
    /// [buttons.action]
    /// type    = "launch"
    /// command = "firefox"
    /// args    = ["--private-window"]
    /// ```
    Launch {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },

    /// Run an arbitrary shell command.
    /// ```toml
    /// [buttons.action]
    /// type    = "script"
    /// command = "notify-send 'Hello!'"
    /// ```
    Script { command: String },

    /// Send a key combination to the active window.
    /// Keys are joined with `+`, e.g. `"ctrl+shift+t"` or `"cmd+space"`.
    /// ```toml
    /// [buttons.action]
    /// type = "keypress"
    /// keys = "ctrl+shift+t"
    /// ```
    Keypress { keys: String },
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read config file {:?}", path))?;
        toml::from_str(&content)
            .with_context(|| format!("Cannot parse config file {:?}", path))
    }
}
