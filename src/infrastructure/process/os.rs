//! Process spawns with a time limit. Excluded from the coverage gate.

use super::child::{kill_group, read_in_background};
use super::failure_message::{emit, exit_message, Logging};
use std::io;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

#[cfg(test)]
#[path = "os_tests.rs"]
mod tests;

/// After the child exits, how long its pipes may still take to drain. A grandchild can hold a
/// pipe open after the child exits. Past this, the pipes are given up.
const READ_GRACE: Duration = Duration::from_millis(100);

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

/// Spawn, wait, and collect both pipes. `Err` on spawn failure. `Ok(None)` on timeout, or when
/// a pipe stays open past the deadline or cannot be read.
fn capture(program: &str, args: &[&str], timeout: Duration) -> io::Result<Option<Output>> {
    let deadline = Instant::now() + timeout;
    let mut cmd = Command::new(program);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    // Its own process group, so a timeout can also end the children it started.
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    let mut child = cmd.spawn()?;
    let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
        kill_group(&mut child);
        return Ok(None);
    };
    // Read both pipes on their own threads. A full pipe would otherwise stall the child until timeout.
    let out_rx = read_in_background(stdout);
    let err_rx = read_in_background(stderr);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                kill_group(&mut child);
                return Ok(None);
            }
        }
    };
    // Never join the readers: a grandchild may hold a pipe open forever. Wait a bounded time.
    let read_until = deadline.max(Instant::now() + READ_GRACE);
    let stdout = out_rx
        .recv_timeout(read_until.saturating_duration_since(Instant::now()))
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok());
    let Some(stdout) = stdout else {
        kill_group(&mut child);
        return Ok(None);
    };
    let stderr = err_rx
        .recv_timeout(read_until.saturating_duration_since(Instant::now()))
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    Ok(Some(Output {
        stdout,
        stderr,
        status,
    }))
}
