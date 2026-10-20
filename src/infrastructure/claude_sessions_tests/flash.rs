use super::*;

#[test]
fn flash_scripts_embed_the_device_path() {
    let [iterm, terminal] = flash_scripts("ttys003").unwrap_or_default();
    assert!(iterm.contains(r#"tty of s is "/dev/ttys003""#));
    assert!(terminal.contains(r#"tty of t is "/dev/ttys003""#));
    assert!(iterm.contains(r#"application "iTerm2""#));
    assert!(terminal.contains(r#"application "Terminal""#));
}

#[test]
fn flash_scripts_refuse_a_tty_that_is_not_a_device_name() {
    assert_eq!(flash_scripts("ttys003\" & x"), None);
    assert_eq!(flash_scripts("pts/1"), None);
}

#[test]
fn flash_scripts_flash_three_times_then_restore() {
    for script in flash_scripts("ttys003").unwrap_or_default() {
        assert!(script.contains("repeat 3 times"));
        assert_eq!(script.matches("to {65535, 45000, 0}").count(), 1);
        let loop_end = script.rfind("end try").unwrap_or(0);
        let restore = script.rfind(" to orig").unwrap_or(0);
        assert!(restore > loop_end, "restore must follow the loop");
        assert!(script.rfind("return \"found\"").unwrap_or(0) > restore);
    }
}

#[test]
fn flash_scripts_save_the_colour_before_changing_it() {
    let [iterm, terminal] = flash_scripts("ttys003").unwrap_or_default();
    assert!(iterm.contains("set orig to background color of s"));
    assert!(terminal.contains("set orig to background color of t"));
}
