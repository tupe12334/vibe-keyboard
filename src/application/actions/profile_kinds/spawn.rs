//! Kinds that start a program: the recorder must see the program and its key args.

use super::{calls, check_tap_and_hold, Rig};
use crate::domain::profile::Action;

pub fn open_url() {
    let action = Action::OpenUrl {
        url: "https://example.com".into(),
    };
    check_tap_and_hold(&action, &["open -- https://example.com"]);
    // A key with a hold runs its tap on a short press, and the hold never starts.
    {
        let mut short = Rig::slow(action, Some(Action::PushToTalk { key: None }));
        short.press();
        short.release();
    }
    assert_eq!(calls(), ["open -- https://example.com"]);
}

pub fn run_command() {
    let action = Action::RunCommand {
        command: "echo hi".into(),
    };
    check_tap_and_hold(&action, &["sh -c echo hi"]);
}

/// A plain name opens with `-a`. A bundle id opens with `-b`.
pub fn open_app() {
    let by_name = Action::OpenApp {
        name: "Safari".into(),
    };
    check_tap_and_hold(&by_name, &["open -a Safari"]);
    let by_bundle = Action::OpenApp {
        name: "com.apple.Terminal".into(),
    };
    check_tap_and_hold(&by_bundle, &["open -b com.apple.Terminal"]);
}

/// A good shortcut runs `osascript`. A bad one records nothing and logs the error.
pub fn hotkey() {
    let good = Action::Hotkey {
        keys: "cmd+shift+4".into(),
    };
    let script =
        "tell application \"System Events\" to keystroke \"4\" using {command down, shift down}";
    let line = format!("osascript -e {script}");
    check_tap_and_hold(&good, &[line.as_str()]);
    let bad = Action::Hotkey {
        keys: "cmd+nope".into(),
    };
    check_tap_and_hold(&bad, &[]);
}
