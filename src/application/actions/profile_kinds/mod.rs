//! Every action kind, pressed through the real key handler with no device.
//! Side effects go to recorders, so no program starts and no key is posted.

use crate::domain::profile::{Action, ActionKind};

mod page;
mod ptt;
mod recorder;
mod rig;
mod spawn;

pub use recorder::calls;
pub use rig::Rig;

/// Run `action` as a tap, then as a hold. Each must record exactly `expected`.
pub fn check_tap_and_hold(action: &Action, expected: &[&str]) {
    {
        let mut tap = Rig::new(action.clone(), None);
        tap.press();
    }
    assert_eq!(calls(), expected, "tap of {action:?}");
    {
        let mut hold = Rig::new(Action::None, Some(action.clone()));
        hold.press();
        hold.poll();
        hold.release();
    }
    assert_eq!(calls(), expected, "hold of {action:?}");
}

/// The scenario for one kind. The match has no wildcard: a new kind fails to compile here.
fn check_kind(kind: ActionKind) {
    match kind {
        ActionKind::None => check_tap_and_hold(&Action::None, &[]),
        ActionKind::OpenUrl => spawn::open_url(),
        ActionKind::RunCommand => spawn::run_command(),
        ActionKind::OpenApp => spawn::open_app(),
        ActionKind::Hotkey => spawn::hotkey(),
        ActionKind::GoToPage => page::go_to_page(),
        ActionKind::Builtin => page::builtin_numpad(),
        ActionKind::PushToTalk => ptt::push_to_talk(),
    }
}

#[test]
fn every_action_kind_runs_through_the_key_handler() {
    for kind in ActionKind::ALL {
        check_kind(kind);
    }
}
