use super::super::*;

#[test]
fn run_call_sends_the_session_when_the_server_sets_one() {
    let mut sent = Vec::new();
    let mut exchange = |body: &str, session: Option<&str>, read: Option<u64>| {
        sent.push((body.to_owned(), session.map(str::to_owned), read));
        match read {
            Some(_) => Some((None, Some("reply".into()))),
            None if session.is_none() => Some((Some("s1".into()), None)),
            None => Some((None, None)),
        }
    };
    assert_eq!(run_call(&mut exchange, "tool", 2).as_deref(), Some("reply"));
    assert_eq!(sent[0].1, None);
    assert_eq!(sent[1].1.as_deref(), Some("s1"));
    assert_eq!(sent[2], ("tool".to_owned(), Some("s1".to_owned()), Some(2)));
}

#[test]
fn run_call_works_with_a_stateless_server() {
    let mut sent = Vec::new();
    let mut exchange = |_: &str, session: Option<&str>, read: Option<u64>| {
        sent.push(session.map(str::to_owned));
        Some((None, read.map(|_| "reply".to_owned())))
    };
    assert_eq!(run_call(&mut exchange, "tool", 2).as_deref(), Some("reply"));
    assert_eq!(sent, vec![None, None, None]);
}

#[test]
fn run_call_stops_when_the_handshake_fails() {
    let mut exchange = |_: &str, _: Option<&str>, _: Option<u64>| None;
    assert_eq!(run_call(&mut exchange, "tool", 2), None);
}

#[test]
fn press_label_shows_failed_unless_the_daemon_answered() {
    assert_eq!(press_label(true, "STARTED"), "STARTED");
    assert_eq!(press_label(false, "STARTED"), "FAILED");
}
