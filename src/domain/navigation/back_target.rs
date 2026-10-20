use super::Screen;

/// What key 11 does on a screen. `Back` steps within the main pages. `Out` leaves a sub-screen,
/// the same as key 12.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackKey {
    Back,
    Out,
}

/// The key 11 decision for `screen`. Main pages keep back. Every built-in sub-screen uses out.
pub const fn back_key_on(screen: &Screen) -> BackKey {
    match screen {
        Screen::MainPage { .. } => BackKey::Back,
        _ => BackKey::Out,
    }
}
