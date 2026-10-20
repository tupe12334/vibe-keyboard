use super::{applescript_string, is_valid_tty};

/// Two `AppleScript`s that type `text` and Enter into the tab on `/dev/<tty>`: `[iTerm2,
/// Terminal.app]`. Each prints `found` on success and `missing` otherwise. None for an invalid tty
/// or text that holds an unsafe control character.
pub fn send_text_scripts(tty: &str, text: &str) -> Option<[String; 2]> {
    if !is_valid_tty(tty) {
        return None;
    }
    let device = format!("/dev/{tty}");
    let text = applescript_string(text)?;
    let iterm = format!(
        r#"if application "iTerm2" is running then
  tell application "iTerm2"
    repeat with w in windows
      repeat with t in tabs of w
        repeat with s in sessions of t
          if tty of s is "{device}" then
            tell s to write text {text}
            return "found"
          end if
        end repeat
      end repeat
    end repeat
  end tell
end if
return "missing""#
    );
    let terminal = format!(
        r#"if application "Terminal" is running then
  tell application "Terminal"
    repeat with w in windows
      repeat with t in tabs of w
        if tty of t is "{device}" then
          do script {text} in t
          return "found"
        end if
      end repeat
    end repeat
  end tell
end if
return "missing""#
    );
    Some([iterm, terminal])
}

/// Two `AppleScript`s that select the tab on `/dev/<tty>`: `[iTerm2, Terminal.app]`.
/// Each prints `found` on success and `missing` otherwise. None for an invalid tty.
pub fn focus_scripts(tty: &str) -> Option<[String; 2]> {
    if !is_valid_tty(tty) {
        return None;
    }
    let device = format!("/dev/{tty}");
    let iterm = format!(
        r#"if application "iTerm2" is running then
  tell application "iTerm2"
    repeat with w in windows
      repeat with t in tabs of w
        repeat with s in sessions of t
          if tty of s is "{device}" then
            tell s to select
            tell t to select
            tell w to select
            set index of w to 1
            activate
            return "found"
          end if
        end repeat
      end repeat
    end repeat
  end tell
end if
return "missing""#
    );
    let terminal = format!(
        r#"if application "Terminal" is running then
  tell application "Terminal"
    repeat with w in windows
      repeat with t in tabs of w
        if tty of t is "{device}" then
          set selected of t to true
          set frontmost of w to true
          set index of w to 1
          activate
          return "found"
        end if
      end repeat
    end repeat
  end tell
end if
return "missing""#
    );
    Some([iterm, terminal])
}
