use std::collections::HashMap;

/// `(pid, tty)` for each `claude` process that has a terminal, from `ps -Ao pid=,tty=,comm=`.
/// Skips `??` (no terminal, for example a background helper).
pub fn claude_processes(ps: &str) -> Vec<(u32, String)> {
    ps.lines()
        .filter_map(|line| {
            let (pid, rest) = line.trim_start().split_once(' ')?;
            let (tty, comm) = rest.trim_start().split_once(' ')?;
            let name = comm.trim().rsplit('/').next()?;
            if name != "claude" || tty == "??" {
                return None;
            }
            Some((pid.parse().ok()?, tty.to_string()))
        })
        .collect()
}

/// Working directory per pid from `lsof -a -d cwd -Fpn -p <pids>`. A `p` line starts a process
/// and its `n` line holds the path.
pub fn cwds_from_lsof(output: &str) -> HashMap<u32, String> {
    let mut cwds = HashMap::new();
    let mut pid: Option<u32> = None;
    for line in output.lines() {
        if let Some(text) = line.strip_prefix('p') {
            pid = text.parse().ok();
        } else if let (Some(id), Some(path)) = (pid, line.strip_prefix('n')) {
            if !path.is_empty() {
                cwds.insert(id, path.to_string());
            }
        }
    }
    cwds
}
