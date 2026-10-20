/// A keyboard shortcut such as `cmd+shift+4`, sent through System Events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hotkey {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Command,
    Shift,
    Option,
    Control,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// A printable character, sent with `keystroke`.
    Char(char),
    /// A macOS virtual key code, sent with `key code`.
    Code(u8),
}

/// Parse `cmd+shift+4` style strings. The last token is the key; the rest are modifiers.
pub fn parse_hotkey(text: &str) -> Result<Hotkey, String> {
    // `split` always yields at least one token, so popping the key cannot fail.
    let mut tokens: Vec<String> = text.split('+').map(|t| t.trim().to_lowercase()).collect();
    let key_token = tokens.pop().unwrap_or_default();
    let mod_tokens = tokens;

    let modifiers = mod_tokens
        .iter()
        .map(|m| match m.as_str() {
            "cmd" | "command" => Ok(Modifier::Command),
            "shift" => Ok(Modifier::Shift),
            "opt" | "option" | "alt" => Ok(Modifier::Option),
            "ctrl" | "control" => Ok(Modifier::Control),
            other => Err(format!("unknown modifier '{other}'")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let key = match key_token.as_str() {
        "return" | "enter" => Key::Code(36),
        "tab" => Key::Code(48),
        "space" => Key::Code(49),
        "delete" | "backspace" => Key::Code(51),
        "escape" | "esc" => Key::Code(53),
        "left" => Key::Code(123),
        "right" => Key::Code(124),
        "down" => Key::Code(125),
        "up" => Key::Code(126),
        _ => char_key(&key_token)?,
    };

    Ok(Hotkey { modifiers, key })
}

mod keycodes;

pub use keycodes::{default_push_to_talk, key_code, modifier_flags, push_to_talk_key};

/// A single character key. Anything longer or empty is an error.
fn char_key(token: &str) -> Result<Key, String> {
    let mut chars = token.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(Key::Char(c)),
        (None, _) => Err("missing key after '+'".to_string()),
        _ => Err(format!("unknown key '{token}'")),
    }
}

impl Hotkey {
    /// `AppleScript` that sends this shortcut via System Events.
    pub fn applescript(&self) -> String {
        let send = match &self.key {
            Key::Char(c) => format!("keystroke \"{}\"", escape_applescript(&c.to_string())),
            Key::Code(code) => format!("key code {code}"),
        };
        if self.modifiers.is_empty() {
            return format!("tell application \"System Events\" to {send}");
        }
        let mods: Vec<&str> = self
            .modifiers
            .iter()
            .map(|m| match m {
                Modifier::Command => "command down",
                Modifier::Shift => "shift down",
                Modifier::Option => "option down",
                Modifier::Control => "control down",
            })
            .collect();
        let mods = mods.join(", ");
        format!("tell application \"System Events\" to {send} using {{{mods}}}")
    }
}

fn escape_applescript(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
#[path = "hotkey_tests.rs"]
mod tests;
