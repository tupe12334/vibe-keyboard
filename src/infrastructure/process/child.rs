//! Pipe readers and process-group teardown for `os.rs`.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};

/// Drain a pipe on its own thread. The bytes arrive on the channel once the pipe closes.
pub(super) fn read_in_background<R: Read + Send + 'static>(mut pipe: R) -> Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if pipe.read_to_end(&mut bytes).is_ok() {
            let _ = tx.send(bytes);
        }
    });
    rx
}

/// Kill the child and every process in its group. Without the group, a daemon the child started
/// keeps running, and it can hold the pipes open.
pub(super) fn kill_group(child: &mut Child) {
    // ponytail: shells out to `kill` because no libc or nix dependency is present.
    let group = format!("-{}", child.id());
    let _ = Command::new("kill")
        .args(["-s", "KILL", "--", &group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
    let _ = child.wait();
}
