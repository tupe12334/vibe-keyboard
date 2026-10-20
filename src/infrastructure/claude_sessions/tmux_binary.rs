//! Pure choice of the `tmux` binary to run.
//!
//! The app starts at login under launchd, where PATH has no Homebrew dir. The bare `tmux` name
//! then fails, so the known install paths are tried after it. The disk checks live in
//! `claude_sessions/os.rs`.

/// Tried in order. The bare name comes first, so a PATH install wins.
pub const TMUX_CANDIDATES: &[&str] = &[
    "tmux",
    "/opt/homebrew/bin/tmux",
    "/usr/local/bin/tmux",
    "/opt/local/bin/tmux",
    "/usr/bin/tmux",
];

/// First candidate that `exists` accepts. `None` when tmux is installed nowhere known.
pub fn pick_tmux(exists: impl Fn(&str) -> bool) -> Option<&'static str> {
    TMUX_CANDIDATES.iter().copied().find(|c| exists(c))
}

#[cfg(test)]
#[path = "tmux_binary_tests.rs"]
mod tests;
