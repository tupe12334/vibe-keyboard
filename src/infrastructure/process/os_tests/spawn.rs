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

/// True while a process whose command line matches `pattern` is alive.
fn sleeper_alive(pattern: &str) -> bool {
    Command::new("pgrep")
        .args(["-f", pattern])
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[test]
fn probe_kills_a_grandchild_that_holds_only_stderr() {
    // stdout reaches EOF, but the background sleep keeps stderr open after the shell exits.
    assert!(probe(
        "sh",
        &["-c", "sleep 37 >/dev/null & exit 0"],
        Duration::from_secs(5)
    )
    .is_some());
    // SIGKILL is near-instant. The pause only lets the killed process finish exiting.
    std::thread::sleep(Duration::from_millis(500));
    assert!(!sleeper_alive("sleep 37"), "sleep 37 survived the probe");
}

#[test]
fn probe_returns_at_its_deadline_when_a_grandchild_holds_the_pipe() {
    // The shell exits 0, but its background sleep keeps stdout open.
    let started = Instant::now();
    let out = probe(
        "sh",
        &["-c", "sleep 30 & exit 0"],
        Duration::from_millis(300),
    );
    assert!(out.is_none());
    assert!(started.elapsed() < Duration::from_secs(5));
}
