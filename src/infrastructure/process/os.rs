//! Process spawns with a time limit. Excluded from the coverage gate.

use super::failure_message::{emit, exit_message, Logging};
use std::io::{self, Read};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

#[cfg(test)]
#[path = "os_tests.rs"]
mod tests;

/// Run a program with no shell. Returns its stdout when it ends within `timeout`. The exit
/// status does not change the result, because `lsof` exits non-zero when one pid has already
/// gone. A non-zero exit is logged at WARN with its first stderr line.
pub fn run(program: &str, args: &[&str], timeout: Duration) -> Option<String> {
    run_with(program, args, timeout, Logging::Warn)
}

/// `run` with the log level and detail chosen by the caller. See `Logging`.
pub fn run_with(
    program: &str,
    args: &[&str],
    timeout: Duration,
    logging: Logging,
) -> Option<String> {
    let out = match capture(program, args, timeout) {
        Ok(Some(out)) => out,
        Ok(None) => return None,
        Err(err) => {
            emit(logging, &format!("{program} failed to spawn: {err}"));
            return None;
        }
    };
    if !out.status.success() {
        emit(
            logging,
            &exit_message(program, out.status.code(), &out.stderr, logging),
        );
    }
    Some(out.stdout)
}

struct Output {
    stdout: String,
    stderr: String,
    status: ExitStatus,
}

/// Spawn, wait, and collect both pipes. `Err` on spawn failure. `Ok(None)` on timeout or when a
/// pipe cannot be read.
fn capture(program: &str, args: &[&str], timeout: Duration) -> io::Result<Option<Output>> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let (Some(mut stdout), Some(mut stderr)) = (child.stdout.take(), child.stderr.take()) else {
        return Ok(None);
    };
    // Read both pipes on their own threads. A full pipe would otherwise stall the child until timeout.
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
                return Ok(None);
            }
        }
    };
    let stderr = err_reader.join().unwrap_or_default();
    let stdout = out_reader.join().ok().and_then(Result::ok);
    Ok(stdout.map(|stdout| Output {
        stdout,
        stderr,
        status,
    }))
}
