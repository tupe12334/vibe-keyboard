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
