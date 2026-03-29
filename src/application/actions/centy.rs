use std::process::Command;

#[allow(clippy::zombie_processes)]
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
