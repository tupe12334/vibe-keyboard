//! Process spawns for `lsappinfo`. Excluded from the coverage gate; the
//! logic that decides what to do with the output lives in `frontmost.rs`.

use super::{parse_bundle_id, parse_pid, Watcher};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// Hard limit for one `lsappinfo` call. A hung `LaunchServices` call must not
/// stall the watcher.
const TIMEOUT: Duration = Duration::from_secs(1);

/// Bundle id of the frontmost app, e.g. `com.google.Chrome`. None if unknown.
pub fn frontmost_bundle_id() -> Option<String> {
    bundle_of(&lsappinfo(&["front"])?)
}

/// Pid of the frontmost app: the app that gets typed keys. None if unknown.
pub fn frontmost_pid() -> Option<i32> {
    parse_pid(&lsappinfo(&[
        "info",
        "-only",
        "pid",
        &lsappinfo(&["front"])?,
    ])?)
}

/// Spawn a thread that polls every `interval` and sends the bundle id on each
/// change. The first value is sent immediately.
///
/// The thread notices a dropped receiver only when it next tries to send, so
/// it exits on the first change after the receiver is gone.
pub fn watch(interval: Duration) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut watcher = Watcher::default();
        loop {
            if let Some(id) = watcher.tick(lsappinfo(&["front"]), bundle_of) {
                if tx.send(id).is_err() {
                    break;
                }
            }
            std::thread::sleep(interval);
        }
    });
    rx
}

fn bundle_of(asn: &str) -> Option<String> {
    parse_bundle_id(&lsappinfo(&["info", "-only", "bundleid", asn])?)
}

#[cfg(target_os = "macos")]
fn lsappinfo(args: &[&str]) -> Option<String> {
    use std::io::Read;
    use std::process::{Command, Stdio};
    use std::time::Instant;

    let mut child = Command::new("lsappinfo")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match child.try_wait().ok()? {
            Some(status) => {
                let mut out = String::new();
                child.stdout.take()?.read_to_string(&mut out).ok()?;
                return status.success().then(|| out.trim().to_string());
            }
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// Off macOS there is no LaunchServices. Every lookup is None.
#[cfg(not(target_os = "macos"))]
fn lsappinfo(_args: &[&str]) -> Option<String> {
    None
}
