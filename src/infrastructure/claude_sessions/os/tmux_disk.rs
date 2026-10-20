//! Disk checks for the tmux candidates. The pure choice lives in `claude_sessions/tmux_binary.rs`.

use std::path::Path;
use std::time::Duration;

use super::super::tmux_binary::pick_tmux;
use crate::infrastructure::process::run;

const TMUX_TIMEOUT: Duration = Duration::from_millis(250);
const TMUX_PANES: &[&str] = &["list-panes", "-a", "-F", "#{session_name} #{pane_tty}"];

/// Pane list from the first tmux binary that exists. Empty when tmux is absent.
pub(super) fn tmux_panes() -> String {
    pick_tmux(tmux_on_disk)
        .and_then(|program| run(program, TMUX_PANES, TMUX_TIMEOUT))
        .unwrap_or_default()
}

/// True when `candidate` runs as tmux. A bare name is searched on PATH. A path is checked directly.
fn tmux_on_disk(candidate: &str) -> bool {
    if candidate.contains('/') {
        return is_executable(Path::new(candidate));
    }
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(candidate)))
    })
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}
