use std::cell::RefCell;

use super::super::session_reads::FrontReads;
use super::*;

/// Fake OS reads for `landed_pid`. Each pid read takes the next value from `pids`. Logs every read.
struct Fake {
    pids: RefCell<Vec<Option<i32>>>,
    bundle: Option<&'static str>,
    shown: bool,
    log: RefCell<Vec<&'static str>>,
}

impl FrontReads for Fake {
    fn front_pid(&self, _: Duration) -> Option<i32> {
        self.log.borrow_mut().push("pid");
        self.pids.borrow_mut().remove(0)
    }
    fn front_bundle(&self, _: Duration) -> Option<String> {
        self.log.borrow_mut().push("bundle");
        self.bundle.map(str::to_string)
    }
    fn tty_shown(&self, _: usize, _: &str, _: Duration) -> bool {
        self.log.borrow_mut().push("tty");
        self.shown
    }
}

fn check(
    pids: Vec<Option<i32>>,
    bundle: Option<&'static str>,
    shown: bool,
) -> (Option<i32>, Vec<&'static str>) {
    let fake = Fake {
        pids: RefCell::new(pids),
        bundle,
        shown,
        log: RefCell::new(Vec::new()),
    };
    let pid = landed_pid(&fake, "ttys001", &|| GATE_DEADLINE);
    (pid, fake.log.into_inner())
}

#[test]
fn terminal_in_front_with_its_tab_gives_the_pid_in_order() {
    let (pid, log) = check(vec![Some(10), Some(10)], Some("com.apple.Terminal"), true);
    assert_eq!(pid, Some(10));
    assert_eq!(log, ["pid", "bundle", "tty", "pid"]);
}

#[test]
fn pid_that_changes_during_the_check_gives_no_pid() {
    let (pid, _) = check(vec![Some(10), Some(11)], Some("com.apple.Terminal"), true);
    assert_eq!(pid, None);
}

#[test]
fn other_app_in_front_stops_before_the_tty_read() {
    let (pid, log) = check(vec![Some(10)], Some("com.google.Chrome"), true);
    assert_eq!(pid, None);
    assert_eq!(log, ["pid", "bundle"]);
}

#[test]
fn terminal_without_the_session_tab_gives_no_pid() {
    let (pid, log) = check(
        vec![Some(10), Some(10)],
        Some("com.googlecode.iterm2"),
        false,
    );
    assert_eq!(pid, None);
    assert_eq!(log, ["pid", "bundle", "tty"]);
}
