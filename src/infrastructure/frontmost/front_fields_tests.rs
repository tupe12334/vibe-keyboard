use std::cell::{Cell, RefCell};
use std::time::Duration;

use super::*;

/// Fake `lsappinfo`: each call costs `cost`, capped at the timeout it is given. Answers `front`
/// with an ASN and any `info` call with `answer`. Records every timeout it was given.
fn fake_run<'a>(
    clock: &'a Cell<Duration>,
    cost: Duration,
    timeouts: &'a RefCell<Vec<Duration>>,
    answer: &'a str,
) -> impl Fn(&[&str], Duration) -> Option<String> + 'a {
    move |args: &[&str], timeout: Duration| {
        timeouts.borrow_mut().push(timeout);
        clock.set(clock.get() + cost.min(timeout));
        Some(if args[0] == "front" {
            "ASN:0-1".to_string()
        } else {
            answer.to_string()
        })
    }
}

#[test]
fn two_reads_stay_within_one_limit() {
    let limit = Duration::from_millis(300);
    let cost = Duration::from_millis(240);
    let clock = Cell::new(Duration::ZERO);
    let timeouts = RefCell::new(Vec::new());
    let run = fake_run(&clock, cost, &timeouts, "\"pid\"=42");
    let elapsed = || clock.get();
    assert_eq!(pid_within(limit, &elapsed, &run), Some(42));
    assert!(clock.get() <= limit, "both reads used {:?}", clock.get());
    assert_eq!(timeouts.borrow().as_slice(), &[limit, limit - cost]);
}

#[test]
fn bundle_read_is_bounded_the_same_way() {
    let limit = Duration::from_millis(300);
    let cost = Duration::from_millis(200);
    let clock = Cell::new(Duration::ZERO);
    let timeouts = RefCell::new(Vec::new());
    let answer = "\"CFBundleIdentifier\"=\"com.apple.Terminal\"";
    let run = fake_run(&clock, cost, &timeouts, answer);
    let elapsed = || clock.get();
    assert_eq!(
        bundle_within(limit, &elapsed, &run),
        Some("com.apple.Terminal".to_string())
    );
    assert!(clock.get() <= limit);
    assert_eq!(timeouts.borrow().as_slice(), &[limit, limit - cost]);
}

#[test]
fn no_front_app_means_no_second_read() {
    let clock = Cell::new(Duration::ZERO);
    let timeouts = RefCell::new(Vec::new());
    let run = |_: &[&str], timeout: Duration| {
        timeouts.borrow_mut().push(timeout);
        None
    };
    let elapsed = || clock.get();
    assert_eq!(pid_within(Duration::from_millis(300), &elapsed, &run), None);
    assert_eq!(timeouts.borrow().len(), 1);
}

#[test]
fn failed_info_read_gives_no_pid() {
    let clock = Cell::new(Duration::ZERO);
    let run = |args: &[&str], _: Duration| (args[0] == "front").then(|| "ASN:0-1".to_string());
    let elapsed = || clock.get();
    assert_eq!(pid_within(Duration::from_millis(300), &elapsed, &run), None);
}
