use std::process::Command;
use tracing::error;

#[allow(
    clippy::zombie_processes,
    reason = "the GUI launcher runs detached and is never waited on"
)]
pub fn open_in_chrome(url: &str) {
    Command::new("open")
        .args(["-a", "Google Chrome", url])
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Chrome for {url}: {e}");
            std::process::exit(1)
        });
}
