//! Reading MCP replies: the JSON of the reply with our request id, and the tool text in it.

use serde_json::{json, Value};

use super::{initialize_body, initialized_body};

/// One HTTP exchange: send `body` with an optional session id. Gives back the session id the
/// server set, and the reply text when `read` names the request id to wait for.
pub type Reply = (Option<String>, Option<String>);

/// The exchange `run_call` drives: one request in, one `Reply` out.
pub type Exchange<'a> = dyn FnMut(&str, Option<&str>, Option<u64>) -> Option<Reply> + 'a;

/// One tool call with its own handshake: initialize, the initialized notification, then the call.
/// A server that issues no session id is stateless: its later calls go out without one.
pub fn run_call(exchange: &mut Exchange<'_>, tool_body: &str, id: u64) -> Option<String> {
    let (session, _) = exchange(&initialize_body(), None, None)?;
    exchange(&initialized_body(), session.as_deref(), None)?;
    exchange(tool_body, session.as_deref(), Some(id)).and_then(|(_, text)| text)
}

/// The JSON text of the reply with request `id`. Reads `lines` only until that reply is complete,
/// so a stream that stays open after the reply does not hold the caller. A plain JSON body is one
/// document. An SSE stream is read event by event: a blank line ends an event, and the `data:`
/// lines of an event join with `\n`.
pub fn reply_text<I: IntoIterator<Item = String>>(lines: I, id: u64) -> Option<String> {
    let mut lines = lines.into_iter();
    let mut event: Vec<String> = Vec::new();
    while let Some(line) = lines.next() {
        if event.is_empty() && line.trim_start().starts_with('{') {
            let mut text = line;
            for rest in lines {
                text.push('\n');
                text.push_str(&rest);
            }
            return matches_id(&text, id).then_some(text);
        }
        if let Some(data) = line.strip_prefix("data:") {
            event.push(data.strip_prefix(' ').unwrap_or(data).to_owned());
        } else if line.trim().is_empty() {
            if let Some(text) = take_event(&mut event, id) {
                return Some(text);
            }
        }
    }
    take_event(&mut event, id)
}

/// The joined data of a finished event, when it is the reply with `id`. Clears the event either way.
fn take_event(event: &mut Vec<String>, id: u64) -> Option<String> {
    let text = event.join("\n");
    event.clear();
    matches_id(&text, id).then_some(text)
}

fn matches_id(text: &str, id: u64) -> bool {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|v| v.get("id").cloned())
        == Some(json!(id))
}

/// The JSON in a reply body. `None` when it is not JSON.
pub fn json_payload(body: &str) -> Option<Value> {
    serde_json::from_str(body.trim()).ok()
}

/// The text of a `tools/call` result. `None` when the body has no result or the tool errored.
pub fn tool_text(body: &str) -> Option<String> {
    let payload = json_payload(body)?;
    let result = payload.get("result")?;
    if result.get("isError").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    result["content"][0]["text"].as_str().map(str::to_owned)
}
