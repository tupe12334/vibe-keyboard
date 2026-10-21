use super::*;
use crate::infrastructure::process::probe;

#[test]
fn probe_failure_logs_at_debug() {
    let log = logged(|| {
        probe("sh", &["-c", "exit 1"], Duration::from_secs(5));
    });
    assert!(
        log.contains("DEBUG") && log.contains("exited with code 1"),
        "{log}"
    );
    assert!(!log.contains("WARN"), "{log}");
}

#[test]
fn success_logs_nothing() {
    assert!(run_logged(Logging::Warn, "sh", &["-c", "exit 0"]).is_empty());
}

#[test]
fn spawn_failure_is_logged_at_the_chosen_level() {
    let log = run_logged(Logging::Debug, "/no/such/vibe-keyboard-program", &[]);
    assert!(
        log.contains("DEBUG") && log.contains("failed to spawn"),
        "{log}"
    );
}
