use serde::{Deserialize, Serialize};

/// The `[claude_sessions]` table of `profile.toml`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaudeSessionsConfig {
    /// Open each new Claude session window in macOS full screen.
    #[serde(default)]
    pub new_session_fullscreen: bool,
}

impl ClaudeSessionsConfig {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}
