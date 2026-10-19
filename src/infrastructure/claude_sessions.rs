//! Finds open Claude Code sessions, their titles and status, and focuses their terminal tab.
//!
//! This file holds the pure logic. Process spawns live in `claude_sessions/os.rs`.

mod os;

pub use os::{focus_session, list_sessions};

use std::collections::HashMap;
use std::time::Duration;

use crate::domain::actions::{ClaudeSession, Glyph, SessionStatus};

/// A tty that printed within this long is `Active`. Older is `Stale`. Claude Code redraws while it
/// works, so a session that is quiet for a minute is waiting or idle.
pub const ACTIVE_WITHIN: Duration = Duration::from_secs(60);

/// A spinner keeps a session `Active` only this long after its tty last printed. Past this, a tab
/// left with a spinner (Claude exited mid-task) reads as `Stale`.
pub const SPINNER_ACTIVE_WITHIN: Duration = Duration::from_mins(10);

const CLAUDE_SUFFIX: &str = " (claude)";
/// Longest session name shown on a key. The key font fits about three short lines.
const SUMMARY_CHARS: usize = 60;

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

/// Short name for a working directory: `~` for home, else the last path component.
pub fn label_for(cwd: &str, home: &str) -> String {
    if cwd == home {
        return "~".into();
    }
    cwd.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("/")
        .to_string()
}

/// True for the spinner glyphs Claude Code cycles while it works: `◐◑◒◓` and braille.
const fn is_spinner(c: char) -> bool {
    matches!(c, '\u{25D0}'..='\u{25D3}' | '\u{2800}'..='\u{28FF}')
}

/// Activity status from the tty age and the title glyph. A spinner means Claude is mid-task, so it
/// is `Active` while the tty is at most `SPINNER_ACTIVE_WITHIN` old. Otherwise the age decides
/// against `ACTIVE_WITHIN`. `✳` does not change the result.
pub fn status_for(age: Duration, title_glyph: Option<Glyph>) -> SessionStatus {
    let spinning = title_glyph == Some(Glyph::Spinner) && age <= SPINNER_ACTIVE_WITHIN;
    if spinning || age <= ACTIVE_WITHIN {
        SessionStatus::Active
    } else {
        SessionStatus::Stale
    }
}

/// Short age for a key corner: `NOW` under a minute, then `2M`, `1H`, `3D`. Empty when unknown.
pub fn age_label(age: Option<Duration>) -> String {
    let Some(age) = age else {
        return String::new();
    };
    let secs = age.as_secs();
    match secs {
        0..=59 => "NOW".into(),
        60..=3599 => format!("{}M", secs / 60),
        3600..=86_399 => format!("{}H", secs / 3600),
        _ => format!("{}D", secs / 86_400),
    }
}

/// Glyph and title from a terminal tab title such as `✳ Centy availability (claude)`.
/// The title keeps no glyph and no ` (claude)` suffix.
pub fn parse_title(raw: &str) -> (Option<Glyph>, String) {
    let raw = raw.trim();
    let (glyph, rest) = match raw.chars().next() {
        Some('✳') => (Some(Glyph::Waiting), &raw['✳'.len_utf8()..]),
        Some(c) if is_spinner(c) => (Some(Glyph::Spinner), &raw[c.len_utf8()..]),
        _ => (None, raw),
    };
    let rest = rest.trim();
    let title = rest.strip_suffix(CLAUDE_SUFFIX).unwrap_or(rest).trim_end();
    (glyph, title.to_string())
}

/// Tab titles keyed by bare tty, from `<device>\t<title>` lines. Lines with a bad tty are dropped.
pub fn parse_tab_titles(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let (device, title) = line.split_once('\t')?;
            let tty = device.strip_prefix("/dev/")?;
            is_valid_tty(tty).then(|| (tty.to_string(), title.to_string()))
        })
        .collect()
}

/// The session name in a Claude Code transcript (`.jsonl`). Uses the last `summary` line, else
/// the first user prompt. The result is cut to `SUMMARY_CHARS`.
pub fn summary_from_jsonl(text: &str) -> Option<String> {
    let mut summary: Option<String> = None;
    let mut first_prompt: Option<String> = None;
    for value in text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
    {
        match value.get("type").and_then(serde_json::Value::as_str) {
            Some("summary") => {
                summary = value
                    .get("summary")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .or(summary);
            }
            // A prompt that is only a pasted link makes a poor session name, so skip it.
            Some("user") if first_prompt.is_none() => {
                first_prompt = user_text(&value).filter(|text| !is_url(text));
            }
            _ => {}
        }
    }
    summary
        .or(first_prompt)
        .map(|text| text.chars().take(SUMMARY_CHARS).collect())
}

fn is_url(text: &str) -> bool {
    let text = text.trim_start();
    text.starts_with("http://") || text.starts_with("https://")
}

/// Prompt text of a user line. Content is either a string or a list of parts with `text`.
fn user_text(value: &serde_json::Value) -> Option<String> {
    match value.pointer("/message/content")? {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(parts) => parts
            .iter()
            .find_map(|part| part.get("text").and_then(serde_json::Value::as_str))
            .map(str::to_string),
        _ => None,
    }
}

/// Build the session list from `(tty, cwd)` pairs. `ages` holds the tty age per tty.
///
/// Title source, first match wins: the terminal tab title, then the transcript summary for the
/// working directory (`summary_for`), then the directory label, then the tty. The status comes
/// from `status_for`. The list is sorted: active first by label, then stale by most recent.
pub fn build_sessions(
    procs: Vec<(String, Option<String>)>,
    tab_titles: &HashMap<String, String>,
    ages: &HashMap<String, Duration>,
    home: &str,
    summary_for: impl Fn(&str) -> Option<String>,
) -> Vec<ClaudeSession> {
    let mut sessions: Vec<ClaudeSession> = procs
        .into_iter()
        .map(|(tty, cwd)| {
            let age = ages.get(&tty).copied();
            let titled = tab_titles
                .get(&tty)
                .map(|raw| parse_title(raw))
                .filter(|(_, title)| !title.is_empty());
            let (glyph, label) = match titled {
                Some(found) => found,
                None => (
                    None,
                    match cwd.as_deref() {
                        Some(path) => summary_for(path).unwrap_or_else(|| label_for(path, home)),
                        None => tty.clone(),
                    },
                ),
            };
            let status = status_for(age.unwrap_or(Duration::MAX), glyph);
            ClaudeSession {
                tty,
                label,
                status,
                age,
            }
        })
        .collect();
    sessions.sort_by(|a, b| {
        order_key(a)
            .cmp(&order_key(b))
            .then_with(|| (&a.label, &a.tty).cmp(&(&b.label, &b.tty)))
    });
    sessions
}

/// Active sessions compare equal, so they sort by label and do not jump on each refresh. Stale
/// sessions sort by age, the most recent first.
fn order_key(session: &ClaudeSession) -> (SessionStatus, Duration) {
    match session.status {
        SessionStatus::Active => (SessionStatus::Active, Duration::ZERO),
        SessionStatus::Stale => (SessionStatus::Stale, session.age.unwrap_or(Duration::MAX)),
    }
}

/// True for `ttysNNN` only. The tty goes into an `AppleScript` string, so anything else is refused.
pub fn is_valid_tty(tty: &str) -> bool {
    tty.strip_prefix("ttys")
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// Two `AppleScript`s that select the tab on `/dev/<tty>`: `[iTerm2, Terminal.app]`.
/// Each prints `found` on success and `missing` otherwise. None for an invalid tty.
pub fn focus_scripts(tty: &str) -> Option<[String; 2]> {
    if !is_valid_tty(tty) {
        return None;
    }
    let device = format!("/dev/{tty}");
    let iterm = format!(
        r#"if application "iTerm2" is running then
  tell application "iTerm2"
    repeat with w in windows
      repeat with t in tabs of w
        repeat with s in sessions of t
          if tty of s is "{device}" then
            tell s to select
            tell t to select
            set index of w to 1
            activate
            return "found"
          end if
        end repeat
      end repeat
    end repeat
  end tell
end if
return "missing""#
    );
    let terminal = format!(
        r#"if application "Terminal" is running then
  tell application "Terminal"
    repeat with w in windows
      repeat with t in tabs of w
        if tty of t is "{device}" then
          set selected of t to true
          set index of w to 1
          activate
          return "found"
        end if
      end repeat
    end repeat
  end tell
end if
return "missing""#
    );
    Some([iterm, terminal])
}

#[cfg(test)]
mod tests {
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
            ("ttys001".to_string(), Some("/a/one".to_string())),
            ("ttys002".to_string(), Some("/a/two".to_string())),
            ("ttys003".to_string(), Some("/a/three".to_string())),
            ("ttys004".to_string(), None),
        ];
        let summary = |cwd: &str| (cwd == "/a/two").then(|| "From summary".to_string());
        let ages = HashMap::from([("ttys001".to_string(), Duration::ZERO)]);
        let got = build_sessions(procs, &titles, &ages, "/Users/me", summary);
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
            ("ttys001".to_string(), None),
            ("ttys002".to_string(), None),
            ("ttys003".to_string(), None),
            ("ttys004".to_string(), None),
            ("ttys005".to_string(), None),
        ];
        let order: Vec<String> = build_sessions(procs, &titles, &ages, "/", |_| None)
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
    fn focus_scripts_refuse_a_bad_tty() {
        assert_eq!(focus_scripts("ttys003\"; quit"), None);
        assert_eq!(focus_scripts("??"), None);
    }
}
