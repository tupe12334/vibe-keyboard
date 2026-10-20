/// Tab titles from iTerm2, one `<device>\t<name>` line per session. Asked only when iTerm2 runs,
/// so this never launches it.
pub(super) const ITERM_TITLES: &str = r#"if application "iTerm2" is running then
  tell application "iTerm2"
    set out to ""
    repeat with w in windows
      repeat with t in tabs of w
        repeat with s in sessions of t
          set out to out & (tty of s) & (character id 9) & (name of s) & linefeed
        end repeat
      end repeat
    end repeat
    return out
  end tell
end if
return """#;

/// Tab titles from Terminal.app, one `<device>\t<custom title>` line per tab. Same guard as iTerm2.
pub(super) const TERMINAL_TITLES: &str = r#"if application "Terminal" is running then
  tell application "Terminal"
    set out to ""
    repeat with w in windows
      repeat with t in tabs of w
        set out to out & (tty of t) & (character id 9) & (custom title of t) & linefeed
      end repeat
    end repeat
    return out
  end tell
end if
return """#;
