//! Decides whether a `claude` session runs under VS Code. The focus fallback opens VS Code only then.

const VSCODE_MARKERS: [&str; 2] = ["Visual Studio Code", "Code Helper"];
// ponytail: a real process tree is far shallower. The cap only stops a malformed table from looping.
const MAX_DEPTH: usize = 64;

/// One row of `ps -Ao pid=,ppid=,tty=,comm=`. `comm` keeps the full path.
struct Row {
    pid: u32,
    ppid: u32,
    tty: String,
    comm: String,
}

/// True when a `claude` on `tty` has a VS Code process among its ancestors. `ps` is the output of
/// `ps -Ao pid=,ppid=,tty=,comm=`.
pub fn is_vscode_session(ps: &str, tty: &str) -> bool {
    let rows: Vec<Row> = ps.lines().filter_map(parse_row).collect();
    rows.iter()
        .filter(|row| row.tty == tty && row.comm.rsplit('/').next() == Some("claude"))
        .any(|row| has_vscode_ancestor(&rows, row.pid))
}

fn parse_row(line: &str) -> Option<Row> {
    let mut fields = line.split_whitespace();
    let pid = fields.next()?.parse().ok()?;
    let ppid = fields.next()?.parse().ok()?;
    let tty = fields.next()?.to_string();
    let comm = fields.collect::<Vec<_>>().join(" ");
    Some(Row {
        pid,
        ppid,
        tty,
        comm,
    })
}

fn has_vscode_ancestor(rows: &[Row], pid: u32) -> bool {
    std::iter::successors(Some(pid), |&id| {
        rows.iter().find(|r| r.pid == id).map(|r| r.ppid)
    })
    .skip(1)
    .take(MAX_DEPTH)
    .filter_map(|id| rows.iter().find(|r| r.pid == id))
    .any(|r| VSCODE_MARKERS.iter().any(|marker| r.comm.contains(marker)))
}
