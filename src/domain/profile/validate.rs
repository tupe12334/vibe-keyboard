use crate::domain::keys::is_nav_key;

use super::{parse_color, Profile, KEY_COUNT, PROMPT_LABEL_MAX_CHARS, PROMPT_MAX_CHARS};

mod key_binding;

impl Profile {
    /// Human-readable problems in the layout. Empty when the profile is valid.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (index, prompt) in self.prompts.iter().enumerate() {
            let place = format!("prompt {}", index + 1);
            if prompt.label.trim().is_empty() {
                out.push(format!("{place}: label is empty"));
            }
            if prompt.label.chars().any(char::is_control) {
                out.push(format!("{place}: label must be one line"));
            }
            if prompt.label.chars().count() > PROMPT_LABEL_MAX_CHARS {
                out.push(format!(
                    "{place}: label is over {PROMPT_LABEL_MAX_CHARS} characters"
                ));
            }
            if prompt.text.trim().is_empty() {
                out.push(format!("{place}: text is empty"));
            }
            if prompt.text.chars().any(char::is_control) {
                out.push(format!("{place}: text must be one line"));
            }
            if prompt.text.chars().count() > PROMPT_MAX_CHARS {
                out.push(format!(
                    "{place}: text is over {PROMPT_MAX_CHARS} characters"
                ));
            }
        }
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
                if let Some(problem) = config
                    .hold
                    .as_ref()
                    .and_then(|h| h.problem(self.pages.len()))
                {
                    out.push(format!("{place}: hold {problem}"));
                }
            }
        }
        out
    }
}
