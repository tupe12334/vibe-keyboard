use super::*;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Shared buffer that the test subscriber writes into.
#[derive(Clone, Default)]
struct Buf(Arc<Mutex<Vec<u8>>>);

impl Write for Buf {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Ok(mut all) = self.0.lock() {
            all.extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Runs `body` under a DEBUG subscriber and returns the log text it wrote.
fn logged(body: impl FnOnce()) -> String {
    let buf = Buf::default();
    let sink = buf.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_writer(move || sink.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, body);
    let bytes = buf.0.lock().map(|all| all.clone()).unwrap_or_default();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn run_logged(logging: Logging, program: &str, args: &[&str]) -> String {
    logged(|| {
        run_with(program, args, Duration::from_secs(5), logging);
    })
}

const SECRET: &str = "tok-SECRET-123";
/// Fails and echoes its argument to stderr, the way a failing `AppleScript` can echo a prompt.
const ECHO_ARG: &[&str] = &[
    "-c",
    "echo \"AppleScript error near $1\" >&2; exit 3",
    "sh",
    SECRET,
];

#[test]
fn warn_logs_code_and_first_stderr_line() {
    let log = run_logged(Logging::Warn, "sh", &["-c", "echo boom >&2; exit 3"]);
    assert!(
        log.contains("WARN") && log.contains("code 3") && log.contains("boom"),
        "{log}"
    );
}

#[test]
fn code_only_never_logs_stderr_or_user_text() {
    let log = run_logged(Logging::WarnCodeOnly, "sh", ECHO_ARG);
    assert!(
        log.contains("WARN") && log.contains("exited with code 3"),
        "{log}"
    );
    assert!(!log.contains(SECRET), "{log}");
}

#[test]
fn echoed_argument_would_leak_under_plain_warn() {
    // Proves the code-only test above is meaningful: the same call does leak under Warn.
    assert!(run_logged(Logging::Warn, "sh", ECHO_ARG).contains(SECRET));
}

#[test]
fn debug_logs_at_debug_level_not_warn() {
    let log = run_logged(Logging::Debug, "sh", &["-c", "echo boom >&2; exit 1"]);
    assert!(log.contains("DEBUG") && log.contains("boom"), "{log}");
    assert!(!log.contains("WARN"), "{log}");
}

#[path = "os_tests/spawn.rs"]
mod spawn;
