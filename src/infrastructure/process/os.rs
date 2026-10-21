//! Process spawns with a time limit. Excluded from the coverage gate.

use super::failure_message::failure_message;
use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

#[cfg(test)]
#[path = "os_tests.rs"]
mod tests;

/// Run a program with no shell. Returns its stdout when it ends within `timeout`. The exit
/// status does not change the result, because `lsof` exits non-zero when one pid has already
/// gone. A non-zero exit is logged at WARN with its first stderr line.
pub fn run(program: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let out = capture(program, args, timeout)?;
    if !out.status.success() {
        tracing::warn!(
            "{}",
            failure_message(program, out.status.code(), &out.stderr)
        );
    }
    Some(out.stdout)
}

struct Output {
    stdout: String,
    stderr: String,
    status: ExitStatus,
}

/// Spawn, wait, and collect both pipes. `None` on spawn failure, timeout, or unreadable stdout.
fn capture(program: &str, args: &[&str], timeout: Duration) -> Option<Output> {
    let spawned = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => {
            tracing::warn!(program, error = %err, "failed to spawn");
            return None;
        }
    };
    // Read both pipes on their own threads. A full pipe would otherwise stall the child until timeout.
    let mut stdout = child.stdout.take()?;
    let mut stderr = child.stderr.take()?;
    let out_reader = std::thread::spawn(move || {
        let mut text = String::new();
        stdout.read_to_string(&mut text).map(|_| text)
    });
    let err_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    let stderr = err_reader.join().unwrap_or_default();
    let stdout = out_reader.join().ok()?.ok()?;
    Some(Output {
        stdout,
        stderr,
        status,
    })
}
