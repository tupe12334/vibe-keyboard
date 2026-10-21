//! The fake side effects. Each call is written as one line: program and args, or the key and hotkey.

use std::cell::RefCell;

use crate::application::actions::effects::Effects;
use crate::domain::hotkey::Hotkey;

/// Generation the fake hold reserves.
const GENERATION: u64 = 7;

thread_local! {
    static CALLS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// The calls the fake effects recorded on this thread, oldest first.
pub fn calls() -> Vec<String> {
    CALLS.with(|calls| calls.borrow().clone())
}

/// Forget every recorded call on this thread.
pub fn clear() {
    CALLS.with(|calls| calls.borrow_mut().clear());
}

fn record(line: String) {
    CALLS.with(|calls| calls.borrow_mut().push(line));
}

fn fake_spawn(program: &str, args: &[&str]) {
    record(format!("{program} {}", args.join(" ")));
}

fn fake_tap(hotkey: &Hotkey) {
    record(format!("tap {hotkey:?}"));
}

fn fake_hold_begin(key: u8) -> u64 {
    record(format!("hold_begin {key}"));
    GENERATION
}

fn fake_hold_end(key: u8) {
    record(format!("hold_end {key}"));
}

fn fake_ptt_start(generation: u64, key: u8, hotkey: &Hotkey, tty: Option<&str>) {
    record(format!(
        "ptt_start gen={generation} key={key} {hotkey:?} tty={tty:?}"
    ));
}

/// No Accessibility grant, so the hotkey path takes its log branch and makes no OS call.
fn fake_trusted() -> bool {
    false
}

pub const FAKE: Effects = Effects {
    spawn: fake_spawn,
    tap: fake_tap,
    hold_begin: fake_hold_begin,
    hold_end: fake_hold_end,
    push_to_talk_start: fake_ptt_start,
    trusted: fake_trusted,
};
