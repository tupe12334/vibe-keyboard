use super::*;

#[test]
fn sessions_prefer_tab_title_then_summary_then_label_then_tty() {
    let mut titles = HashMap::new();
    titles.insert("ttys001".to_string(), "◐ Titled (claude)".to_string());
    titles.insert("ttys002".to_string(), "   ".to_string());
    let procs = vec![
        (101, "ttys001".to_string(), Some("/a/one".to_string())),
        (102, "ttys002".to_string(), Some("/a/two".to_string())),
        (103, "ttys003".to_string(), Some("/a/three".to_string())),
        (104, "ttys004".to_string(), None),
    ];
    let summary = |cwd: &str| (cwd == "/a/two").then(|| "From summary".to_string());
    let ages = HashMap::from([("ttys001".to_string(), Duration::ZERO)]);
    let got = build_sessions(procs, &titles, &HashMap::new(), &ages, "/Users/me", summary);
    let got: Vec<(&str, &str, SessionStatus)> = got
        .iter()
        .map(|s| (s.tty.as_str(), s.label.as_str(), s.status))
        .collect();
    assert_eq!(
        got,
        vec![
            ("ttys001", "Titled", SessionStatus::Active),
            ("ttys002", "From summary", SessionStatus::Stale),
            ("ttys003", "three", SessionStatus::Stale),
            ("ttys004", "ttys004", SessionStatus::Stale),
        ]
    );
}

#[test]
fn sessions_record_pid_and_owning_app() {
    let mut owners = HashMap::new();
    owners.insert("ttys001".to_string(), TerminalApp::ITerm);
    owners.insert("ttys002".to_string(), TerminalApp::Terminal);
    let procs = vec![
        (11, "ttys001".to_string(), None),
        (22, "ttys002".to_string(), None),
        (33, "ttys009".to_string(), None),
    ];
    let got = build_sessions(
        procs,
        &HashMap::new(),
        &owners,
        &HashMap::new(),
        "/",
        |_| None,
    );
    let pairs: Vec<(u32, TerminalApp)> = got.iter().map(|s| (s.pid, s.app)).collect();
    assert_eq!(
        pairs,
        vec![
            (11, TerminalApp::ITerm),
            (22, TerminalApp::Terminal),
            (33, TerminalApp::Other),
        ]
    );
}

#[test]
fn sessions_sort_active_by_label_then_stale_by_recency() {
    let mut titles = HashMap::new();
    titles.insert("ttys001".to_string(), "◐ Zed (claude)".to_string());
    titles.insert("ttys002".to_string(), "✳ Beta (claude)".to_string());
    titles.insert("ttys003".to_string(), "✳ Alpha (claude)".to_string());
    titles.insert("ttys005".to_string(), "✳ Gamma (claude)".to_string());
    let ages: HashMap<String, Duration> = [
        ("ttys001", 0),
        ("ttys002", 500),
        ("ttys003", 100),
        ("ttys004", 900),
        ("ttys005", 10),
    ]
    .into_iter()
    .map(|(tty, secs)| (tty.to_string(), Duration::from_secs(secs)))
    .collect();
    let procs = vec![
        (1, "ttys001".to_string(), None),
        (2, "ttys002".to_string(), None),
        (3, "ttys003".to_string(), None),
        (4, "ttys004".to_string(), None),
        (5, "ttys005".to_string(), None),
    ];
    let order: Vec<String> = build_sessions(procs, &titles, &HashMap::new(), &ages, "/", |_| None)
        .into_iter()
        .map(|s| s.label)
        .collect();
    // Active (Zed spinning, Gamma at 10 s) by label, then stale 100 s, 500 s, 900 s.
    assert_eq!(order, ["Gamma", "Zed", "Alpha", "Beta", "ttys004"]);
}
