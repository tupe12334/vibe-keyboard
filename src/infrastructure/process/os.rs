//! Process spawns with a time limit. Excluded from the coverage gate.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Run a program with no shell. Returns its stdout when it ends within `timeout`. The exit
/// status is ignored, because `lsof` exits non-zero when one pid has already gone.
pub fn run(program: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    // Read stdout on its own thread. A full pipe would otherwise stall the child until timeout.
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        stdout.read_to_string(&mut text).map(|_| text)
    });
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    reader.join().ok()?.ok()
}
