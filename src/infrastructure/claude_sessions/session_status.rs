use std::time::Duration;

use crate::domain::actions::{Glyph, SessionStatus};

use super::{ACTIVE_WITHIN, SPINNER_ACTIVE_WITHIN};

/// Short name for a working directory: `~` for home, else the last path component.
pub fn label_for(cwd: &str, home: &str) -> String {
    if cwd == home {
        return "~".into();
    }
    cwd.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("/")
        .to_string()
}

/// True for the spinner glyphs Claude Code cycles while it works: `◐◑◒◓` and braille.
pub(super) const fn is_spinner(c: char) -> bool {
    matches!(c, '\u{25D0}'..='\u{25D3}' | '\u{2800}'..='\u{28FF}')
}

/// Activity status from the tty age and the title glyph. A spinner means Claude is mid-task, so it
/// is `Active` while the tty is at most `SPINNER_ACTIVE_WITHIN` old. Otherwise the age decides
/// against `ACTIVE_WITHIN`. `✳` does not change the result.
pub fn status_for(age: Duration, title_glyph: Option<Glyph>) -> SessionStatus {
    let spinning = title_glyph == Some(Glyph::Spinner) && age <= SPINNER_ACTIVE_WITHIN;
    if spinning || age <= ACTIVE_WITHIN {
        SessionStatus::Active
    } else {
        SessionStatus::Stale
    }
}

/// Short age for a key corner: `NOW` under a minute, then `2M`, `1H`, `3D`. Empty when unknown.
pub fn age_label(age: Option<Duration>) -> String {
    let Some(age) = age else {
        return String::new();
    };
    let secs = age.as_secs();
    match secs {
        0..=59 => "NOW".into(),
        60..=3599 => format!("{}M", secs / 60),
        3600..=86_399 => format!("{}H", secs / 3600),
        _ => format!("{}D", secs / 86_400),
    }
}
