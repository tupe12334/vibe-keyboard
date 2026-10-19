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
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers_and_char_key() {
        assert_eq!(
            parse_hotkey("cmd+shift+4").map(|h| (h.modifiers, h.key)),
            Ok((vec![Modifier::Command, Modifier::Shift], Key::Char('4')))
        );
    }

    #[test]
    fn parses_named_keys_and_aliases() {
        assert_eq!(
            parse_hotkey("ctrl+option+return").map(|h| h.key),
            Ok(Key::Code(36))
        );
        assert_eq!(parse_hotkey("alt+esc").map(|h| h.key), Ok(Key::Code(53)));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse_hotkey("hyper+a").is_err());
        assert!(parse_hotkey("cmd+nope").is_err());
        assert!(parse_hotkey("cmd+").is_err());
        assert!(parse_hotkey("cmd+ab").is_err());
        assert!(parse_hotkey("+a").is_err());
        assert!(parse_hotkey("").is_err());
    }

    #[test]
    fn applescript_for_char_and_code() {
        assert_eq!(
            parse_hotkey("cmd+shift+4").map(|h| h.applescript()),
            Ok("tell application \"System Events\" to keystroke \"4\" using {command down, shift down}".to_string())
        );
        assert_eq!(
            parse_hotkey("return").map(|h| h.applescript()),
            Ok("tell application \"System Events\" to key code 36".to_string())
        );
    }

    #[test]
    fn parses_every_named_key_and_modifier_alias() {
        for (text, code) in [
            ("tab", 48),
            ("space", 49),
            ("delete", 51),
            ("backspace", 51),
            ("left", 123),
            ("right", 124),
            ("down", 125),
            ("up", 126),
        ] {
            assert_eq!(parse_hotkey(text).map(|h| h.key), Ok(Key::Code(code)));
        }
        assert_eq!(
            parse_hotkey("command+opt+control+tab").map(|h| h.applescript()),
            Ok("tell application \"System Events\" to key code 48 using {command down, option down, control down}".to_string())
        );
    }

    #[test]
    fn escapes_quotes_in_character_keys() {
        assert_eq!(
            parse_hotkey("\"").map(|h| h.applescript()),
            Ok("tell application \"System Events\" to keystroke \"\\\"\"".to_string())
        );
    }
}
