use super::*;

#[test]
fn claude_processes_keeps_only_claude_on_a_terminal() {
    let ps = "  101 ttys003 /usr/local/bin/claude\n\
              \x20 102 ??      claude\n\
              \x20 103 ttys004 zsh\n\
              \x20 104 ttys005 /Users/me/.local/bin/claude\n\
              \x20 105 ttys006 /Applications/Claude Code/claude\n\
              \x20 bad line\n\
              \x20 xyz ttys007 claude\n\
              \x20 106 ttys008 claude-helper\n";
    assert_eq!(
        claude_processes(ps),
        vec![
            (101, "ttys003".to_string()),
            (104, "ttys005".to_string()),
            (105, "ttys006".to_string()),
        ]
    );
}

#[test]
fn claude_processes_skips_lines_without_three_fields() {
    assert!(claude_processes("123 ttys001").is_empty());
    assert!(claude_processes("").is_empty());
}

#[test]
fn cwds_map_each_pid_to_its_n_line() {
    let out = "p4242\nfcwd\nn/Users/me/dev/app\np77\nfcwd\nn/tmp\np9\nfcwd\nn\n";
    let cwds = cwds_from_lsof(out);
    assert_eq!(
        cwds.get(&4242).map(String::as_str),
        Some("/Users/me/dev/app")
    );
    assert_eq!(cwds.get(&77).map(String::as_str), Some("/tmp"));
    assert_eq!(cwds.get(&9), None);
    assert!(cwds_from_lsof("n/orphan\n").is_empty());
}
