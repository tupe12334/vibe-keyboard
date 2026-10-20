//! Hides Moadim routine agents from the session list. A routine runs `claude` in a workbench
//! folder, or in a tmux session that Moadim names `moadim-<routine>`. No deck user drives it.

use std::collections::{HashMap, HashSet};

const WORKBENCH_DIR: &str = "/.moadim/workbenches/";
const MOADIM_SESSION_PREFIX: &str = "moadim-";

/// Drops Moadim routine agents from `procs` (`(pid, tty)`). `cwds` comes from lsof. `tmux` is the
/// output of `tmux list-panes -a -F '#{session_name} #{pane_tty}'`, empty when tmux is absent.
pub fn drop_routines(
    procs: Vec<(u32, String)>,
    cwds: &HashMap<u32, String>,
    tmux: &str,
) -> Vec<(u32, String)> {
    let moadim_ttys = moadim_ttys(tmux);
    procs
        .into_iter()
        .filter(|(pid, tty)| {
            let in_workbench = cwds
                .get(pid)
                .is_some_and(|path| path.contains(WORKBENCH_DIR));
            !(in_workbench || moadim_ttys.contains(tty.as_str()))
        })
        .collect()
}

/// Ttys of tmux panes whose session name starts with `moadim-`. Each line is `<name> <pane_tty>`.
/// The tty may carry a `/dev/` prefix. The name is split off at the last space.
fn moadim_ttys(tmux: &str) -> HashSet<&str> {
    tmux.lines()
        .filter_map(|line| line.rsplit_once(' '))
        .filter(|(name, _)| name.starts_with(MOADIM_SESSION_PREFIX))
        .map(|(_, tty)| tty.trim_start_matches("/dev/"))
        .collect()
}
