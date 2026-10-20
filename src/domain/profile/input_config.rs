use serde::{Deserialize, Serialize};

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
