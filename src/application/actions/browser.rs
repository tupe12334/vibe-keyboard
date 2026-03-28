use std::process::Command;
use tracing::error;

pub fn open_in_browser(url: &str) {
    Command::new("open").arg(url).spawn().unwrap_or_else(|e| {
        error!("Failed to open browser for {url}: {e}");
        std::process::exit(1)
    });
}

pub fn open_in_chrome(url: &str) {
    Command::new("open")
        .args(["-a", "Google Chrome", url])
        .spawn()
        .unwrap_or_else(|e| {
            error!("Failed to open Chrome for {url}: {e}");
            std::process::exit(1)
        });
}
