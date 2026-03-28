use std::process::Command;
use tracing::error;

pub fn open_in_browser(url: &str) {
    Command::new("open").arg(url).spawn().unwrap_or_else(|e| {
        error!("Failed to open browser for {url}: {e}");
        std::process::exit(1)
    });
}
