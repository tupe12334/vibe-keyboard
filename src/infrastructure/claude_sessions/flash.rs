use super::is_valid_tty;

/// Flash colour in `AppleScript` RGB, where 65535 is full scale. A warm amber that reads on dark themes.
const FLASH: &str = "{65535, 45000, 0}";

/// Two `AppleScript`s that flash the pane on `/dev/<tty>`: `[iTerm2, Terminal.app]`. Each sets the
/// flash colour three times, holding 150 ms each time. The original colour is restored in a step
/// after the loop, which runs even when a set inside the loop fails. Each prints `found` on
/// success and `missing` otherwise. None for an invalid tty.
pub fn flash_scripts(tty: &str) -> Option<[String; 2]> {
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
{}
            return "found"
          end if
        end repeat
      end repeat
    end repeat
  end tell
end if
return "missing""#,
        flash_body("s")
    );
    let terminal = format!(
        r#"if application "Terminal" is running then
  tell application "Terminal"
    repeat with w in windows
      repeat with t in tabs of w
        if tty of t is "{device}" then
{}
          return "found"
        end if
      end repeat
    end repeat
  end tell
end if
return "missing""#,
        flash_body("t")
    );
    Some([iterm, terminal])
}

/// The flash for one tab or session `target`. Saves the colour, flashes it, then restores it.
fn flash_body(target: &str) -> String {
    format!(
        r#"            set orig to background color of {target}
            try
              repeat 3 times
                set background color of {target} to {FLASH}
                delay 0.15
                set background color of {target} to orig
                delay 0.15
              end repeat
            end try
            set background color of {target} to orig"#
    )
}
