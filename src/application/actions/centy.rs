use std::process::Command;

#[allow(
    clippy::zombie_processes,
    reason = "fire-and-forget launcher: the spawned app outlives this binary"
)]
pub fn open_centy_workspace(issue_number: u64) {
    let _ = Command::new("pnpm")
        .args([
            "dlx",
            "centy",
            "workspace",
            "open",
            &issue_number.to_string(),
            "--editor",
            "terminal",
        ])
        .spawn();
}
