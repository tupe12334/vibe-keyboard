use super::super::*;

#[test]
fn json_payload_reads_plain_json_only() {
    assert_eq!(json_payload(r#"{"a":1}"#), Some(json!({"a": 1})));
    assert_eq!(json_payload("plain text"), None);
}

/// Lines of a reply as the HTTP reader yields them.
fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_owned).collect()
}

/// A stream the test must not read past. The reader stops at the reply, so reaching this panics.
fn kept_open() -> impl Iterator<Item = String> {
    std::iter::from_fn(|| panic!("read past the reply"))
}

/// Fixture: a notification, then a `tools/call` reply whose JSON is split over two `data:` lines.
const SPLIT_EVENT: &str = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}\n\nevent: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\ndata: \"result\":{\"content\":[{\"type\":\"text\",\"text\":\"ok\"}]}}\n\n";

#[test]
fn reply_text_joins_split_data_lines() {
    let text = reply_text(lines(SPLIT_EVENT), 2).unwrap_or_default();
    assert_eq!(tool_text(&text).as_deref(), Some("ok"));
}

#[test]
fn reply_text_stops_at_the_reply_and_ignores_the_open_stream_after_it() {
    let head = lines("data: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{}}\n\n");
    let text = reply_text(head.into_iter().chain(kept_open()), 2);
    assert_eq!(
        text.as_deref(),
        Some(r#"{"jsonrpc":"2.0","id":2,"result":{}}"#)
    );
}

#[test]
fn reply_text_skips_other_ids_and_returns_none_without_a_match() {
    let other = lines("data: {\"id\":7,\"result\":{}}\n\n");
    assert_eq!(reply_text(other, 2), None);
    let truncated = lines("data: {\"id\":2,\"result\":");
    assert_eq!(reply_text(truncated, 2), None);
    let at_eof = lines("data: {\"id\":2,\"result\":{}}");
    assert!(reply_text(at_eof, 2).is_some());
}

#[test]
fn reply_text_reads_a_plain_json_body_whole() {
    let body = lines("{\"jsonrpc\":\"2.0\",\n\"id\":2,\"result\":{}}");
    assert_eq!(
        reply_text(body, 2).as_deref(),
        Some("{\"jsonrpc\":\"2.0\",\n\"id\":2,\"result\":{}}")
    );
}

#[test]
fn tool_text_reads_result_and_skips_errors() {
    let ok = r#"{"result":{"content":[{"type":"text","text":"hi"}],"isError":false}}"#;
    assert_eq!(tool_text(ok).as_deref(), Some("hi"));
    let failed = r#"{"result":{"content":[{"type":"text","text":"no"}],"isError":true}}"#;
    assert_eq!(tool_text(failed), None);
    assert_eq!(tool_text(r#"{"error":{}}"#), None);
    assert_eq!(tool_text(r#"{"result":{}}"#), None);
}
