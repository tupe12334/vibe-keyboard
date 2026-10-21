use std::process::Command;

/// Runs `git` and returns its trimmed stdout. `None` when git is missing or fails (e.g. a tarball).
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[allow(
    clippy::print_stdout,
    reason = "cargo reads build script stdout for its directives"
)]
fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");

    let sha = git(&["rev-parse", "--short", "HEAD"]).filter(|s| !s.is_empty());
    let sha = match sha {
        Some(sha) if git(&["status", "--porcelain"]).is_some_and(|s| !s.is_empty()) => {
            format!("{sha}-dirty")
        }
        Some(sha) => sha,
        None => "unknown".to_string(),
    };

    let date = Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map_or_else(
            || "unknown".to_string(),
            |o| String::from_utf8_lossy(&o.stdout).trim().to_string(),
        );

    println!("cargo:rustc-env=VK_GIT_SHA={sha}");
    println!("cargo:rustc-env=VK_BUILD_DATE={date}");
}
