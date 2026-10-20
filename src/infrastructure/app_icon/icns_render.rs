//! Render an app icon `.icns` to PNG with `sips`, and run macOS tools under a timeout.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tracing::debug;

/// Limit for each macOS tool call.
const TOOL_TIMEOUT: Duration = Duration::from_secs(3);

/// Render `icns` to a square PNG at `out`. Writes a temp file first and renames it,
/// so a killed run leaves no partial PNG at the cache path.
pub(super) fn render_icns(icns: &Path, size: u32, out: &Path) -> bool {
    let Some(dir) = out.parent() else {
        return false;
    };
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let tmp = out.with_extension(format!("png.{}.part", std::process::id()));
    let side = size.to_string();
    let rendered = run_capture(
        Command::new("sips")
            .args([
                "-s",
                "format",
                "png",
                "--resampleHeightWidth",
                side.as_str(),
                side.as_str(),
            ])
            .arg(icns)
            .arg("--out")
            .arg(&tmp),
    )
    .is_some();
    if rendered && std::fs::rename(&tmp, out).is_ok() {
        return true;
    }
    let _ = std::fs::remove_file(&tmp);
    false
}

/// Run a tool, collect its stdout, and kill it after `TOOL_TIMEOUT`.
/// Returns `None` on spawn failure, non-zero exit, or timeout.
pub(super) fn run_capture(cmd: &mut Command) -> Option<Vec<u8>> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    // Read on a thread so a large output cannot block the child on a full pipe.
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).map(|_| buf)
    });
    let deadline = Instant::now() + TOOL_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().ok()? {
            let buf = reader.join().ok()?.ok()?;
            return status.success().then_some(buf);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            debug!("tool timed out, killed");
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
