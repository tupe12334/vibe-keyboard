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

#[test]
fn label_is_home_tilde_or_last_component() {
    assert_eq!(label_for("/Users/me", "/Users/me"), "~");
    assert_eq!(label_for("/Users/me/dev/app", "/Users/me"), "app");
    assert_eq!(label_for("/Users/me/dev/app/", "/Users/me"), "app");
    assert_eq!(label_for("/", "/Users/me"), "/");
}

#[test]
fn glyph_is_read_and_removed_with_the_suffix() {
    assert_eq!(
        parse_title("✳ Centy availability (claude)"),
        (Some(Glyph::Waiting), "Centy availability".to_string())
    );
    assert_eq!(
        parse_title("◐ USB devices overview (claude)"),
        (Some(Glyph::Spinner), "USB devices overview".to_string())
    );
    assert_eq!(
        parse_title("⠋ Spinning"),
        (Some(Glyph::Spinner), "Spinning".to_string())
    );
    assert_eq!(parse_title("  zsh  "), (None, "zsh".to_string()));
}

#[test]
fn status_is_active_within_a_minute_or_when_spinning() {
    let quiet = Duration::from_secs(61);
    assert_eq!(status_for(Duration::ZERO, None), SessionStatus::Active);
    assert_eq!(
        status_for(Duration::from_secs(60), None),
        SessionStatus::Active
    );
    assert_eq!(status_for(quiet, None), SessionStatus::Stale);
    assert_eq!(
        status_for(quiet, Some(Glyph::Waiting)),
        SessionStatus::Stale
    );
    assert_eq!(
        status_for(quiet, Some(Glyph::Spinner)),
        SessionStatus::Active
    );
}

#[test]
fn spinner_keeps_active_only_within_ten_minutes() {
    let nine_min = Duration::from_mins(9);
    let eleven_min = Duration::from_mins(11);
    assert_eq!(
        status_for(nine_min, Some(Glyph::Spinner)),
        SessionStatus::Active
    );
    assert_eq!(
        status_for(eleven_min, Some(Glyph::Spinner)),
        SessionStatus::Stale
    );
    assert_eq!(
        status_for(Duration::MAX, Some(Glyph::Spinner)),
        SessionStatus::Stale
    );
}

#[test]
fn age_label_units() {
    assert_eq!(age_label(None), "");
    assert_eq!(age_label(Some(Duration::from_secs(5))), "NOW");
    assert_eq!(age_label(Some(Duration::from_secs(120))), "2M");
    assert_eq!(age_label(Some(Duration::from_secs(3600))), "1H");
    assert_eq!(age_label(Some(Duration::from_hours(72))), "3D");
}

#[test]
fn tab_titles_need_a_valid_device_path() {
    let out = "/dev/ttys003\t✳ Alpha (claude)\n/dev/ttys00x\tbad\nttys004\tno device\n\
               /dev/ttys005\t\n";
    let titles = parse_tab_titles(out);
    assert_eq!(
        titles.get("ttys003").map(String::as_str),
        Some("✳ Alpha (claude)")
    );
    assert_eq!(titles.get("ttys005").map(String::as_str), Some(""));
    assert_eq!(titles.len(), 2);
}

#[test]
fn summary_is_the_last_summary_else_the_first_prompt() {
    let text = concat!(
        "not json\n",
        r#"{"type":"user","message":{"content":"Fix the login bug please, it fails"}}"#,
        "\n",
        r#"{"type":"summary","summary":"Old name"}"#,
        "\n",
        r#"{"type":"summary","summary":"Login fix"}"#,
        "\n",
    );
    assert_eq!(summary_from_jsonl(text), Some("Login fix".to_string()));
    let prompt_only = concat!(
        r#"{"type":"user","message":{"content":[{"type":"text","text":"Build the thing"}]}}"#,
        "\n",
        r#"{"type":"user","message":{"content":"second prompt"}}"#,
        "\n",
    );
    assert_eq!(
        summary_from_jsonl(prompt_only),
        Some("Build the thing".to_string())
    );
    let long = format!(
        r#"{{"type":"user","message":{{"content":"{}"}}}}"#,
        "x".repeat(200)
    );
    assert_eq!(
        summary_from_jsonl(&long).map(|s| s.chars().count()),
        Some(SUMMARY_CHARS)
    );
}

#[test]
fn real_tab_line_gives_the_title_and_url_prompts_are_skipped() {
    let out = "/dev/ttys000\t✳ Centy availability (claude)\n";
    let titles = parse_tab_titles(out);
    assert_eq!(
        titles.get("ttys000").map(String::as_str),
        Some("✳ Centy availability (claude)")
    );
    let url_first = concat!(
        r#"{"type":"user","message":{"content":"https://example.com/pasted"}}"#,
        "\n",
        r#"{"type":"user","message":{"content":"Write the report"}}"#,
        "\n",
    );
    assert_eq!(
        summary_from_jsonl(url_first),
        Some("Write the report".to_string())
    );
    let url_only = r#"{"type":"user","message":{"content":"http://x.test"}}"#;
    assert_eq!(summary_from_jsonl(url_only), None);
}

#[test]
fn user_line_without_text_gives_no_summary() {
    let tool_only = r#"{"type":"user","message":{"content":[{"type":"tool_result"}]}}"#;
    let numeric = r#"{"type":"user","message":{"content":42}}"#;
    assert_eq!(summary_from_jsonl(tool_only), None);
    assert_eq!(summary_from_jsonl(numeric), None);
    assert_eq!(summary_from_jsonl(r#"{"type":"assistant"}"#), None);
}

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

#[test]
fn tty_must_be_ttys_and_digits() {
    assert!(is_valid_tty("ttys003"));
    assert!(is_valid_tty("ttys0"));
    assert!(!is_valid_tty("ttys"));
    assert!(!is_valid_tty("ttys00a"));
    assert!(!is_valid_tty("ttys003\" & do shell script \"x"));
    assert!(!is_valid_tty("/dev/ttys003"));
    assert!(!is_valid_tty("??"));
    assert!(!is_valid_tty(""));
}

#[test]
fn focus_scripts_embed_the_device_path() {
    let [iterm, terminal] = focus_scripts("ttys003").unwrap_or_default();
    assert!(iterm.contains(r#"tty of s is "/dev/ttys003""#));
    assert!(terminal.contains(r#"tty of t is "/dev/ttys003""#));
    assert!(iterm.contains(r#"application "iTerm2""#));
    assert!(terminal.contains(r#"application "Terminal""#));
}

#[test]
fn applescript_string_escapes_quotes_and_backslashes() {
    assert_eq!(applescript_string("/recap").as_deref(), Some(r#""/recap""#));
    assert_eq!(
        applescript_string(r#"say "hi""#).as_deref(),
        Some(r#""say \"hi\"""#)
    );
    assert_eq!(applescript_string(r"a\b").as_deref(), Some(r#""a\\b""#));
    assert_eq!(
        applescript_string("héllo ✳").as_deref(),
        Some("\"héllo ✳\"")
    );
}

#[test]
fn applescript_string_joins_newline_cr_and_tab_as_constants() {
    assert_eq!(
        applescript_string("a\nb").as_deref(),
        Some(r#"("a" & linefeed & "b")"#)
    );
    assert_eq!(
        applescript_string("\r").as_deref(),
        Some(r#"("" & return & "")"#)
    );
    assert_eq!(
        applescript_string("x\t\"y\\\n").as_deref(),
        Some(r#"("x" & tab & "\"y\\" & linefeed & "")"#)
    );
}

#[test]
fn applescript_string_refuses_other_control_chars() {
    assert_eq!(applescript_string("a\u{0}b"), None);
    assert_eq!(applescript_string("\u{7f}"), None);
    assert_eq!(applescript_string("\u{1b}[31m"), None);
    assert_eq!(send_text_scripts("ttys001", "a\u{7}b"), None);
}

#[test]
fn send_text_scripts_joins_multiline_text() {
    let [iterm, _] = send_text_scripts("ttys007", "one\ntwo").unwrap_or_default();
    assert!(iterm.contains(r#"write text ("one" & linefeed & "two")"#));
}

#[test]
fn is_session_on_tty_needs_the_same_pid_and_tty() {
    let ps = "  101 ttys003 claude\n  202 ttys004 claude\n";
    assert!(is_session_on_tty(ps, 101, "ttys003"));
    assert!(!is_session_on_tty(ps, 101, "ttys004"), "pid moved tty");
    assert!(!is_session_on_tty(ps, 999, "ttys003"), "pid gone");
    assert!(!is_session_on_tty("", 101, "ttys003"));
}

#[test]
fn send_text_scripts_embed_escaped_text_and_device() {
    let [iterm, terminal] = send_text_scripts("ttys007", r#"a"b"#).unwrap_or_default();
    assert!(iterm.contains(r#"tty of s is "/dev/ttys007""#));
    assert!(iterm.contains(r#"write text "a\"b""#));
    assert!(terminal.contains(r#"tty of t is "/dev/ttys007""#));
    assert!(terminal.contains(r#"do script "a\"b" in t"#));
    assert_eq!(send_text_scripts("ttys007\"; quit", "/recap"), None);
}

#[test]
fn focus_scripts_refuse_a_bad_tty() {
    assert_eq!(focus_scripts("ttys003\"; quit"), None);
    assert_eq!(focus_scripts("??"), None);
}
