//! HTTP calls to the Moadim daemon. Excluded from the coverage gate. The request bodies and the
//! parsing live in `moadim.rs`.

use std::io::{BufRead, BufReader};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;
use tracing::error;

use super::{
    action_call, parse_routines, reply_text, run_call, tool_call_body, tool_text, Reply,
    RoutineAction,
};
use crate::domain::moadim::Routine;

const URL: &str = "http://localhost:5784/mcp";

/// Longest wait for one daemon request. Past this the routines page reads the daemon as offline.
const TIMEOUT: Duration = Duration::from_secs(3);

/// One HTTP post. Gives back the session id from the response headers, and the reply text when
/// `read` names the request id to wait for. Notifications and the handshake pass `None`.
fn post(body: &str, session: Option<&str>, read: Option<u64>) -> Option<Reply> {
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .build(),
    );
    let mut request = agent
        .post(URL)
        .header("Accept", "application/json, text/event-stream")
        .header("Content-Type", "application/json");
    if let Some(id) = session {
        request = request.header("mcp-session-id", id);
    }
    let mut response = request.send(body).ok()?;
    let session = response
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let Some(id) = read else {
        return Some((session, None));
    };
    let lines = BufReader::new(response.body_mut().as_reader())
        .lines()
        .map_while(Result::ok);
    Some((session, reply_text(lines, id)))
}

/// One tool call with its own handshake. See `run_call`.
fn call(tool_body: &str, id: u64) -> Option<String> {
    run_call(&mut post, tool_body, id)
}

/// Routines the daemon lists for this machine. `None` when the daemon does not answer in time.
pub fn list_routines() -> Option<Vec<Routine>> {
    parse_routines(&call(&tool_call_body(2, "list_routines", &json!({})), 2)?)
}

/// Run `action` on routine `id`. True when the daemon answered without a tool error.
pub fn call_action(action: RoutineAction, id: &str) -> bool {
    let (tool, args) = action_call(action, id, now_secs());
    let ok = call(&tool_call_body(2, tool, &args), 2)
        .and_then(|text| tool_text(&text))
        .is_some();
    if !ok {
        error!("moadim: {tool} failed for routine {id}");
    }
    ok
}

/// Unix seconds now.
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
