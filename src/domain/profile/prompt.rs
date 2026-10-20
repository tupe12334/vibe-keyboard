use serde::{Deserialize, Serialize};

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
