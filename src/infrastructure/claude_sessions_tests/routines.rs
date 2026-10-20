use super::*;

// ps -Ao pid=,tty=,comm= : three claude processes on terminals.
const PS: &str = "  301 ttys011 claude\n\
                  \x20 401 ttys012 claude\n\
                  \x20 501 ttys013 claude\n";

// `tmux list-panes -a -F '#{session_name} #{pane_tty}'`: a Moadim routine on ttys011, a user
// session named "work" on ttys012. ttys013 has no tmux pane.
const TMUX: &str = "moadim-canary-check-1792442588_94847 /dev/ttys011\n\
                    work /dev/ttys012\n";

fn kept_pids(kept: &[(u32, String)]) -> Vec<u32> {
    kept.iter().map(|(pid, _)| *pid).collect()
}

#[test]
fn moadim_tmux_pane_is_hidden() {
    let kept = drop_routines(claude_processes(PS), &HashMap::new(), TMUX);
    assert!(!kept_pids(&kept).contains(&301), "moadim-* pane hidden");
}

#[test]
fn user_tmux_session_and_plain_claude_stay_visible() {
    let kept = drop_routines(claude_processes(PS), &HashMap::new(), TMUX);
    assert_eq!(kept_pids(&kept), vec![401, 501]);
}

#[test]
fn workbench_cwd_is_hidden() {
    let mut cwds = HashMap::new();
    cwds.insert(401, "/Users/me/.moadim/workbenches/canary-1".to_string());
    let kept = drop_routines(claude_processes(PS), &cwds, "");
    assert_eq!(kept_pids(&kept), vec![301, 501]);
}

#[test]
fn empty_tmux_output_applies_only_the_workbench_rule() {
    let kept = drop_routines(claude_processes(PS), &HashMap::new(), "");
    assert_eq!(kept_pids(&kept), vec![301, 401, 501]);
}

#[test]
fn workbench_cwd_needs_the_dot_moadim_folder() {
    let mut cwds = HashMap::new();
    cwds.insert(9, "/Users/me/moadim/workbenches/x".to_string());
    let procs = vec![(9, "ttys001".to_string())];
    assert_eq!(drop_routines(procs, &cwds, "").len(), 1);
}

#[test]
fn tmux_name_must_start_with_moadim_dash() {
    let tmux = "moadim-x /dev/ttys001\n\
                my-moadim-x /dev/ttys002\n\
                moadim_y /dev/ttys003\n\
                moadim /dev/ttys004\n";
    let procs = vec![
        (1, "ttys001".to_string()),
        (2, "ttys002".to_string()),
        (3, "ttys003".to_string()),
        (4, "ttys004".to_string()),
    ];
    let kept = drop_routines(procs, &HashMap::new(), tmux);
    assert_eq!(kept_pids(&kept), vec![2, 3, 4]);
}

#[test]
fn bare_tty_name_without_dev_prefix_matches() {
    let procs = vec![(1, "ttys001".to_string())];
    let kept = drop_routines(procs, &HashMap::new(), "moadim-x ttys001\n");
    assert!(kept.is_empty());
}
