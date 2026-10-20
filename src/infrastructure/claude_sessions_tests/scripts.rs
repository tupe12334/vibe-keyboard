use super::*;

#[test]
fn focus_scripts_embed_the_device_path() {
    let [iterm, terminal] = focus_scripts("ttys003").unwrap_or_default();
    assert!(iterm.contains(r#"tty of s is "/dev/ttys003""#));
    assert!(terminal.contains(r#"tty of t is "/dev/ttys003""#));
    assert!(iterm.contains(r#"application "iTerm2""#));
    assert!(terminal.contains(r#"application "Terminal""#));
}

#[test]
fn applescript_string_escapes_quotes_and_backslashes() {
    assert_eq!(applescript_string("/recap").as_deref(), Some(r#""/recap""#));
    assert_eq!(
        applescript_string(r#"say "hi""#).as_deref(),
        Some(r#""say \"hi\"""#)
    );
    assert_eq!(applescript_string(r"a\b").as_deref(), Some(r#""a\\b""#));
    assert_eq!(
        applescript_string("héllo ✳").as_deref(),
        Some("\"héllo ✳\"")
    );
}

#[test]
fn applescript_string_joins_newline_cr_and_tab_as_constants() {
    assert_eq!(
        applescript_string("a\nb").as_deref(),
        Some(r#"("a" & linefeed & "b")"#)
    );
    assert_eq!(
        applescript_string("\r").as_deref(),
        Some(r#"("" & return & "")"#)
    );
    assert_eq!(
        applescript_string("x\t\"y\\\n").as_deref(),
        Some(r#"("x" & tab & "\"y\\" & linefeed & "")"#)
    );
}

#[test]
fn applescript_string_refuses_other_control_chars() {
    assert_eq!(applescript_string("a\u{0}b"), None);
    assert_eq!(applescript_string("\u{7f}"), None);
    assert_eq!(applescript_string("\u{1b}[31m"), None);
    assert_eq!(send_text_scripts("ttys001", "a\u{7}b"), None);
}

#[test]
fn send_text_scripts_joins_multiline_text() {
    let [iterm, _] = send_text_scripts("ttys007", "one\ntwo").unwrap_or_default();
    assert!(iterm.contains(r#"write text ("one" & linefeed & "two")"#));
}

#[test]
fn send_text_scripts_empty_text_types_only_enter() {
    let [iterm, terminal] = send_text_scripts("ttys007", "").unwrap_or_default();
    assert!(iterm.contains(r#"write text """#));
    assert!(terminal.contains(r#"do script "" in t"#));
}

#[test]
fn send_text_scripts_embed_escaped_text_and_device() {
    let [iterm, terminal] = send_text_scripts("ttys007", r#"a"b"#).unwrap_or_default();
    assert!(iterm.contains(r#"tty of s is "/dev/ttys007""#));
    assert!(iterm.contains(r#"write text "a\"b""#));
    assert!(terminal.contains(r#"tty of t is "/dev/ttys007""#));
    assert!(terminal.contains(r#"do script "a\"b" in t"#));
    assert_eq!(send_text_scripts("ttys007\"; quit", "/recap"), None);
}

#[test]
fn focus_scripts_refuse_a_bad_tty() {
    assert_eq!(focus_scripts("ttys003\"; quit"), None);
    assert_eq!(focus_scripts("??"), None);
}
