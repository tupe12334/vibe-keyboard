//! The side effects a profile key starts: programs, key taps and push-to-talk holds.
//! Production runs them for real. Tests install recorders, so no program starts and no key is posted.

use std::cell::Cell;
use std::process::{Command, Stdio};
use std::thread;

use tracing::error;

use crate::domain::hotkey::Hotkey;
use crate::infrastructure::keyboard;

/// One set of side effects, as plain function pointers.
#[derive(Clone, Copy)]
pub struct Effects {
    pub spawn: fn(&str, &[&str]),
    pub tap: fn(&Hotkey),
    pub hold_begin: fn(u8) -> u64,
    pub hold_end: fn(u8),
    pub push_to_talk_start: fn(u64, u8, &Hotkey, Option<&str>),
}

const REAL: Effects = Effects {
    spawn: spawn_detached,
    tap: keyboard::tap,
    hold_begin: keyboard::hold_begin,
    hold_end: keyboard::hold_end,
    push_to_talk_start: keyboard::push_to_talk_start,
};

thread_local! {
    static CURRENT: Cell<Effects> = const { Cell::new(REAL) };
}

/// The effects this thread runs. Every thread starts with the real ones.
pub fn current() -> Effects {
    CURRENT.with(Cell::get)
}

/// Run `effects` on this thread until the returned guard drops.
#[cfg(test)]
pub fn install(effects: Effects) -> Restore {
    Restore(CURRENT.with(|c| c.replace(effects)))
}

/// Puts the previous effects back when dropped.
#[cfg(test)]
pub struct Restore(Effects);

#[cfg(test)]
impl Drop for Restore {
    fn drop(&mut self) {
        CURRENT.with(|c| c.set(self.0));
    }
}

/// Start a program without waiting for it. A background thread reaps the child.
fn spawn_detached(program: &str, args: &[&str]) {
    match Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            thread::spawn(move || {
                if let Err(e) = child.wait() {
                    error!("wait failed: {e}");
                }
            });
        }
        Err(e) => error!("failed to start {program}: {e}"),
    }
}
