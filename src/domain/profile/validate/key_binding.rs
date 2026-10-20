use crate::domain::hotkey::{key_code, parse_hotkey};

use crate::domain::profile::{Action, BUILTIN_NAMES};

impl Action {
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
            Self::PushToTalk { key } => key.as_deref().and_then(push_to_talk_problem),
        }
    }
}

/// Why a push-to-talk key cannot be posted. `None` when it can.
fn push_to_talk_problem(text: &str) -> Option<String> {
    match parse_hotkey(text) {
        Err(e) => Some(e),
        Ok(hotkey) => key_code(&hotkey.key)
            .is_none()
            .then(|| format!("push-to-talk key '{text}' has no key code")),
    }
}
