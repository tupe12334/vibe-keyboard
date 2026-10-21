//! Remote Control state of a session, from its `~/.claude/sessions/<pid>.json`.
//!
//! Claude Code writes `bridgeSessionId` when the Remote Control bridge connects, and writes `null`
//! when it closes. The file can hold bytes after the JSON object, so only the first value is read.

use crate::domain::actions::ClaudeSession;

/// True when the first JSON value in `text` has a non-empty `bridgeSessionId` string.
pub fn remote_control_on(text: &str) -> bool {
    let first = serde_json::Deserializer::from_str(text)
        .into_iter::<serde_json::Value>()
        .next();
    matches!(first, Some(Ok(value)) if value
        .get("bridgeSessionId")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|id| !id.is_empty()))
}

/// Sets `remote_control` on each session. `read` returns the session file text for a pid, or None.
pub fn mark_remote_control(
    mut sessions: Vec<ClaudeSession>,
    read: impl Fn(u32) -> Option<String>,
) -> Vec<ClaudeSession> {
    for session in &mut sessions {
        session.remote_control = read(session.pid).is_some_and(|text| remote_control_on(&text));
    }
    sessions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::{SessionStatus, TerminalApp};

    #[test]
    fn bridge_id_means_remote_control_is_on() {
        let text = r#"{"pid":24987,"entrypoint":"sdk-cli","bridgeSessionId":"session_01AUJS7HS5"}"#;
        assert!(remote_control_on(text));
    }

    #[test]
    fn null_or_empty_or_missing_means_off() {
        assert!(!remote_control_on(r#"{"pid":1,"bridgeSessionId":null}"#));
        assert!(!remote_control_on(r#"{"pid":1,"bridgeSessionId":""}"#));
        assert!(!remote_control_on(r#"{"pid":1,"entrypoint":"cli"}"#));
    }

    #[test]
    fn trailing_bytes_after_the_object_are_ignored() {
        let text = concat!(
            r#"{"pid":55235,"bridgeSessionId":"session_014FAeoJ"}"#,
            r#"01,"waitingFor":"permission prompt"}"#
        );
        assert!(remote_control_on(text));
        let off = r#"{"pid":55235,"status":"busy"}01,"waitingFor":"x"}"#;
        assert!(!remote_control_on(off));
    }

    #[test]
    fn each_session_takes_its_own_file() {
        let session = |pid| ClaudeSession {
            pid,
            app: TerminalApp::Other,
            tty: "ttys001".into(),
            label: "s".into(),
            status: SessionStatus::Active,
            age: None,
            focused: false,
            remote_control: false,
        };
        let read = |pid: u32| match pid {
            1 => Some(r#"{"bridgeSessionId":"session_a"}"#.to_string()),
            2 => Some(r#"{"bridgeSessionId":null}"#.to_string()),
            _ => None,
        };
        let marked = mark_remote_control(vec![session(1), session(2), session(3)], read);
        let flags: Vec<bool> = marked.iter().map(|s| s.remote_control).collect();
        assert_eq!(flags, [true, false, false]);
    }

    #[test]
    fn broken_or_empty_text_means_off() {
        assert!(!remote_control_on(""));
        assert!(!remote_control_on("not json"));
        assert!(!remote_control_on(r#"{"bridgeSessionId":"session_x""#));
    }
}
