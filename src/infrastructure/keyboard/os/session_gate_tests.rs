use std::cell::Cell;

use super::*;

/// One fake OS read. It takes `cost` on the clock, but never more than its `limit`, as a real
/// read is cut off at its timeout. Returns the pid it saw, if any.
fn fake_read(
    clock: &Cell<Duration>,
    cost: Duration,
    limit: Duration,
    pid: Option<i32>,
) -> Option<i32> {
    clock.set(clock.get() + cost.min(limit));
    pid
}

/// Runs the gate on a fake clock. Returns the pid, how many reads ran, and the largest
/// limit any read was given.
fn run_gate(
    read: impl Fn(&Cell<Duration>, Duration) -> Option<i32>,
    clock: &Cell<Duration>,
) -> (Option<i32>, usize, Duration) {
    let calls = Cell::new(0);
    let max_limit = Cell::new(Duration::ZERO);
    let elapsed = || clock.get();
    let landed = |left: &dyn Fn() -> Duration| {
        calls.set(calls.get() + 1);
        let limit = left();
        max_limit.set(max_limit.get().max(limit));
        read(clock, limit)
    };
    let pause = |step: Duration| clock.set(clock.get() + step);
    let pid = first_landed_pid(GATE_DEADLINE, GATE_STEP, &elapsed, &landed, &pause);
    (pid, calls.get(), max_limit.get())
}

#[test]
fn slow_reads_stop_the_gate_at_the_deadline() {
    let clock = Cell::new(Duration::ZERO);
    let slow = Duration::from_millis(500);
    let (pid, calls, max_limit) =
        run_gate(|clock, limit| fake_read(clock, slow, limit, None), &clock);
    assert_eq!(pid, None, "a wrong app in front posts no key");
    assert_eq!(calls, 1, "one slow read uses up the budget, so no retry");
    assert_eq!(
        max_limit, GATE_DEADLINE,
        "the first read gets the full budget"
    );
    assert_eq!(
        clock.get(),
        GATE_DEADLINE,
        "the read is cut off at the deadline"
    );
}

#[test]
fn fast_failing_reads_never_pass_the_deadline() {
    let clock = Cell::new(Duration::ZERO);
    let quick = Duration::from_millis(100);
    let (pid, calls, _) = run_gate(|clock, limit| fake_read(clock, quick, limit, None), &clock);
    assert_eq!(pid, None);
    assert!(calls >= 2, "several quick tries fit in the budget");
    assert_eq!(
        clock.get(),
        GATE_DEADLINE,
        "the gate stops exactly at the deadline"
    );
}

#[test]
fn gate_takes_the_first_pid_that_lands() {
    let clock = Cell::new(Duration::ZERO);
    let quick = Duration::from_millis(10);
    let (pid, calls, _) = run_gate(
        |clock, limit| {
            let seen = (clock.get() + quick >= GATE_STEP * 2).then_some(42);
            fake_read(clock, quick, limit, seen)
        },
        &clock,
    );
    assert_eq!(pid, Some(42));
    assert!(calls >= 2);
}

#[test]
fn pid_that_changes_before_the_reread_is_refused() {
    assert_eq!(same_pid(10, Some(11)), None, "another app took focus");
    assert_eq!(same_pid(10, None), None, "no pid on the re-read");
    assert_eq!(same_pid(10, Some(10)), Some(10));
}

#[test]
fn only_the_two_terminal_bundles_pass_the_gate() {
    assert_eq!(terminal_index(Some("com.googlecode.iterm2")), Some(0));
    assert_eq!(terminal_index(Some("com.apple.Terminal")), Some(1));
    assert_eq!(terminal_index(Some("com.google.Chrome")), None);
    assert_eq!(terminal_index(None), None);
}
