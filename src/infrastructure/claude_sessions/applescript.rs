//! Guards for text that goes into an `AppleScript` string.

/// True for `ttysNNN` only. The tty goes into an `AppleScript` string, so anything else is refused.
pub fn is_valid_tty(tty: &str) -> bool {
    tty.strip_prefix("ttys")
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// `AppleScript` expression for `text`. Escapes backslash and double quote. `\n`, `\r` and `\t`
/// become `linefeed`, `return` and `tab` joined by `&`. Any other control character gives None.
pub fn applescript_string(text: &str) -> Option<String> {
    let mut out = String::from("\"");
    let mut joined = false;
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\" & linefeed & \""),
            '\r' => out.push_str("\" & return & \""),
            '\t' => out.push_str("\" & tab & \""),
            c if c.is_control() => return None,
            c => out.push(c),
        }
        joined |= matches!(c, '\n' | '\r' | '\t');
    }
    out.push('"');
    Some(if joined { format!("({out})") } else { out })
}
