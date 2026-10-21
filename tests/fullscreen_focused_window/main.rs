//! FULL SCREEN must toggle the focused iTerm2 window, not another one. Opens two real iTerm2
//! windows, so it runs only when `VK_UI_TEST=1` (CI sets it). Needs Accessibility for System Events.
#![cfg(target_os = "macos")]

mod ax;
mod iterm;

use ax::{ax_windows, focused_title, fullscreen_of, wait_fullscreen};
use iterm::{app_toggle_script, open_windows, osa, CloseWindows, FIRST, SECOND};

#[test]
fn full_screen_toggles_only_the_focused_iterm_window() {
    if std::env::var_os("VK_UI_TEST").is_none() {
        return;
    }
    let _close = CloseWindows;
    open_windows();
    let focused = focused_title();
    assert!(
        focused.contains(SECOND),
        "SECOND is not focused, focused is {focused}"
    );
    let order = ax_windows();
    assert!(
        !order.lines().next().is_some_and(|l| l.contains(SECOND)),
        "focused window is AX window 1, so window 1 cannot be told apart:\n{order}"
    );
    let script = app_toggle_script();
    osa(&script);
    let after = wait_fullscreen(SECOND, true);
    assert_eq!(
        fullscreen_of(&after, SECOND),
        Some(true),
        "focused window stayed windowed:\n{after}"
    );
    assert_eq!(
        fullscreen_of(&after, FIRST),
        Some(false),
        "another window went full screen:\n{after}"
    );
    osa(&script);
    let back = wait_fullscreen(SECOND, false);
    assert_eq!(
        fullscreen_of(&back, SECOND),
        Some(false),
        "focused window did not leave full screen:\n{back}"
    );
}
