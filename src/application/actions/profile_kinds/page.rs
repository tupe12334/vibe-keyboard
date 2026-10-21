//! Kinds that move the app: they start no program, so the check is the screen they leave.

use super::{calls, Rig};
use crate::domain::navigation::Screen;
use crate::domain::profile::Action;

/// Tap and hold both move to page 2 and record no effect.
pub fn go_to_page() {
    {
        let mut tap = Rig::new(Action::GoToPage { page: 1 }, None);
        tap.press();
        assert!(matches!(tap.nav.current(), Screen::MainPage { page: 1 }));
    }
    assert!(calls().is_empty(), "tap of GoToPage recorded {:?}", calls());
    {
        let mut hold = Rig::new(Action::None, Some(Action::GoToPage { page: 1 }));
        hold.press();
        hold.poll();
        assert!(matches!(hold.nav.current(), Screen::MainPage { page: 1 }));
        hold.release();
    }
    assert!(
        calls().is_empty(),
        "hold of GoToPage recorded {:?}",
        calls()
    );
}

/// The `numpad` built-in opens the number pad. Other built-ins are not on this path.
pub fn builtin_numpad() {
    let action = Action::Builtin {
        name: "numpad".into(),
    };
    {
        let mut tap = Rig::new(action.clone(), None);
        tap.press();
        assert!(matches!(tap.nav.current(), Screen::InputNumber { .. }));
    }
    {
        let mut hold = Rig::new(Action::None, Some(action));
        hold.press();
        hold.poll();
        assert!(matches!(hold.nav.current(), Screen::InputNumber { .. }));
        hold.release();
    }
    assert!(calls().is_empty(), "numpad recorded {:?}", calls());
}
