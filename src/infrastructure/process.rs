//! Runs a program with a time limit. The spawn lives in `process/os.rs`, which the coverage gate
//! skips. Shared by the Claude sessions and the focused session code.

mod failure_message;
mod os;

use std::time::Duration;

pub use failure_message::Logging;
pub use os::{run, run_with};

/// `run` for a routine probe that fails often, such as a refresh poll. A non-zero exit logs at
/// DEBUG, not WARN.
pub fn probe(program: &str, args: &[&str], timeout: Duration) -> Option<String> {
    run_with(program, args, timeout, Logging::Debug)
}
