//! The Moadim daemon over MCP streamable HTTP (`http://localhost:5784/mcp`). This file holds the
//! pure logic: request bodies, SSE reading and parsing. The HTTP calls live in `moadim/os.rs`.

mod os;
mod reply;
mod routines;

pub use os::{call_action, list_routines, now_secs};
pub use reply::{json_payload, reply_text, run_call, tool_text, Exchange, Reply};
pub use routines::{ago, last_run_label, parse_routines};

use serde_json::{json, Value};

/// How long the SNOOZE key skips scheduled runs, in seconds.
pub const SNOOZE_SECS: u64 = 3600;

/// An action the device sends to one routine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoutineAction {
    Trigger,
    SetEnabled(bool),
    Snooze,
}

/// JSON-RPC `initialize` body. Its response carries the session id for later calls.
pub fn initialize_body() -> String {
    rpc(
        1,
        "initialize",
        &json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "vibe-keyboard", "version": env!("CARGO_PKG_VERSION")}
        }),
    )
}

/// The notification that ends the handshake. It has no id and no response.
pub fn initialized_body() -> String {
    json!({"jsonrpc": "2.0", "method": "notifications/initialized"}).to_string()
}

fn rpc(id: u64, method: &str, params: &Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

/// `tools/call` body that runs tool `name` with `arguments`.
pub fn tool_call_body(id: u64, name: &str, arguments: &Value) -> String {
    rpc(
        id,
        "tools/call",
        &json!({"name": name, "arguments": arguments}),
    )
}

/// The tool name and arguments for `action` on routine `id`. `now` is unix seconds.
pub fn action_call(action: RoutineAction, id: &str, now: u64) -> (&'static str, Value) {
    match action {
        RoutineAction::Trigger => ("trigger_routine", json!({"id": id})),
        RoutineAction::SetEnabled(enabled) => {
            ("update_routine", json!({"id": id, "enabled": enabled}))
        }
        RoutineAction::Snooze => (
            "snooze_routine",
            json!({"id": id, "snoozed_until": now + SNOOZE_SECS}),
        ),
    }
}

/// The label a finished routine action shows: `success` when the daemon answered, else `FAILED`.
pub const fn press_label(ok: bool, success: &'static str) -> &'static str {
    if ok {
        success
    } else {
        "FAILED"
    }
}

#[cfg(test)]
mod tests;
