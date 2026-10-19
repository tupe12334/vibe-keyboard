//! Finds open Claude Code sessions and focuses their terminal tab.
//!
//! This file holds the pure logic. Process spawns live in `claude_sessions/os.rs`.

mod os;

pub use os::{focus_session, list_sessions};

use crate::domain::actions::ClaudeSession;

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

/// The working directory from `lsof -a -p <pid> -d cwd -Fn` output. Its `n` line holds the path.
pub fn cwd_from_lsof(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| line.strip_prefix('n'))
        .filter(|path| !path.is_empty())
        .map(str::to_string)
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

/// Build the session list from `(tty, cwd)` pairs. Pairs without a cwd are dropped.
/// The result is sorted by label, then tty.
pub fn sessions_from(entries: Vec<(String, Option<String>)>, home: &str) -> Vec<ClaudeSession> {
    let mut sessions: Vec<ClaudeSession> = entries
        .into_iter()
        .filter_map(|(tty, cwd)| {
            Some(ClaudeSession {
                tty,
                label: label_for(&cwd?, home),
            })
        })
        .collect();
    sessions.sort_by(|a, b| a.label.cmp(&b.label).then_with(|| a.tty.cmp(&b.tty)));
    sessions
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
    fn cwd_is_the_n_line() {
        let out = "p4242\nfcwd\nn/Users/me/dev/app\n";
        assert_eq!(cwd_from_lsof(out), Some("/Users/me/dev/app".to_string()));
    }

    #[test]
    fn cwd_missing_or_empty_is_none() {
        assert_eq!(cwd_from_lsof("p1\nfcwd\n"), None);
        assert_eq!(cwd_from_lsof("n\n"), None);
    }

    #[test]
    fn label_is_home_tilde_or_last_component() {
        assert_eq!(label_for("/Users/me", "/Users/me"), "~");
        assert_eq!(label_for("/Users/me/dev/app", "/Users/me"), "app");
        assert_eq!(label_for("/Users/me/dev/app/", "/Users/me"), "app");
        assert_eq!(label_for("/", "/Users/me"), "/");
    }

    #[test]
    fn sessions_drop_missing_cwd_and_sort_by_label_then_tty() {
        let entries = vec![
            ("ttys009".to_string(), Some("/a/zeta".to_string())),
            ("ttys002".to_string(), None),
            ("ttys005".to_string(), Some("/a/alpha".to_string())),
            ("ttys004".to_string(), Some("/b/alpha".to_string())),
            ("ttys001".to_string(), Some("/Users/me".to_string())),
        ];
        let got: Vec<(String, String)> = sessions_from(entries, "/Users/me")
            .into_iter()
            .map(|s| (s.label, s.tty))
            .collect();
        let want = [
            ("alpha", "ttys004"),
            ("alpha", "ttys005"),
            ("zeta", "ttys009"),
            ("~", "ttys001"),
        ];
        let want: Vec<(String, String)> = want
            .iter()
            .map(|(l, t)| (l.to_string(), t.to_string()))
            .collect();
        assert_eq!(got, want);
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
