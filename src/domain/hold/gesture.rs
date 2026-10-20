//! What a key does when it comes up or is held, as the tracker reports it.

use crate::domain::profile::Action;

/// What a key does, captured at press time so a later screen change cannot alter it.
#[derive(Debug, Clone, PartialEq)]
pub enum Held {
    /// A main page key. `short` runs on a tap. `hold` runs for the length of a hold.
    Action { short: Action, hold: Option<Action> },
    /// A Claude session key. Its tap focuses the session at press time, so it has no tap gesture.
    Session { tty: String },
    /// The voice key on a session's action page. A hold pushes to talk. A tap sends nothing.
    Voice,
}

/// One outcome of a key. A hold start always comes before its hold end.
#[derive(Debug, Clone, PartialEq)]
pub enum Gesture {
    Tap { key: u8, held: Held },
    HoldStart { key: u8, held: Held },
    HoldEnd { key: u8, held: Held },
}
