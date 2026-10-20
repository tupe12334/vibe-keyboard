use std::collections::HashMap;

/// One row of `ps -Ao pid=,ppid=,tty=,comm=`. `comm` keeps the full path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proc {
    pub pid: u32,
    pub ppid: u32,
    pub tty: String,
    pub comm: String,
}

impl Proc {
    /// The basename of `comm`, for example `claude` for `/Users/me/.local/bin/claude`.
    pub fn name(&self) -> &str {
        self.comm.rsplit('/').next().unwrap_or_default()
    }
}

/// Parses `ps -Ao pid=,ppid=,tty=,comm=`. The command may hold spaces. Lines that do not parse are skipped.
pub fn process_table(ps: &str) -> Vec<Proc> {
    ps.lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pid = fields.next()?.parse().ok()?;
            let ppid = fields.next()?.parse().ok()?;
            let tty = fields.next()?.to_string();
            let comm = fields.collect::<Vec<_>>().join(" ");
            Some(Proc {
                pid,
                ppid,
                tty,
                comm,
            })
        })
        .collect()
}

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
