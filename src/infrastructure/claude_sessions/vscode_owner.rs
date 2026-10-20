//! Decides whether a `claude` session runs under VS Code. The focus fallback opens VS Code only then.

use super::process_list::{process_table, Proc};

const VSCODE_MARKERS: [&str; 2] = ["Visual Studio Code", "Code Helper"];
// ponytail: a real process tree is far shallower. The cap only stops a malformed table from looping.
const MAX_DEPTH: usize = 64;

/// True when a `claude` on `tty` has a VS Code process among its ancestors. `ps` is the output of
/// `ps -Ao pid=,ppid=,tty=,comm=`.
pub fn is_vscode_session(ps: &str, tty: &str) -> bool {
    let procs = process_table(ps);
    procs
        .iter()
        .filter(|proc| proc.tty == tty && proc.name() == "claude")
        .any(|proc| has_vscode_ancestor(&procs, proc.pid))
}

fn has_vscode_ancestor(procs: &[Proc], pid: u32) -> bool {
    std::iter::successors(Some(pid), |&id| {
        procs.iter().find(|p| p.pid == id).map(|p| p.ppid)
    })
    .skip(1)
    .take(MAX_DEPTH)
    .filter_map(|id| procs.iter().find(|p| p.pid == id))
    .any(|p| VSCODE_MARKERS.iter().any(|marker| p.comm.contains(marker)))
}
