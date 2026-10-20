use std::time::Duration;

/// The terminal app that opens a new Claude Code window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewSessionTerminal {
    ITerm,
    Terminal,
}

/// Activity of a Claude Code session. Green is `Active`, orange is `Stale`. Declared in sort order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionStatus {
    /// Output on the tty within the last minute, or a spinner on the tab title.
    Active,
    /// No output for over a minute.
    Stale,
}

/// The status glyph Claude Code puts at the start of a terminal tab title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// `✳`: idle, waiting for the user.
    Waiting,
    /// A spinner glyph: working.
    Spinner,
}

/// Owner of a session's tab. Only iTerm2 and Terminal.app take typed text; others only focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalApp {
    ITerm,
    Terminal,
    Other,
}

/// A Claude Code process on a terminal. `pid` and `tty` identify it. `tty` is the bare name, e.g.
/// `ttys003`. `age` is the time since the tty last printed, or `None` when the tty could not be read.
#[derive(Clone, Debug, PartialEq)]
pub struct ClaudeSession {
    pub pid: u32,
    pub app: TerminalApp,
    pub tty: String,
    pub label: String,
    pub status: SessionStatus,
    pub age: Option<Duration>,
    pub focused: bool,
}
