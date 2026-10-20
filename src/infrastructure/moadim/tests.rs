use super::*;

mod reply;
mod routines;
mod run;

#[test]
fn bodies_are_jsonrpc_requests() {
    let init: Value = serde_json::from_str(&initialize_body()).unwrap_or_default();
    assert_eq!(init["method"], "initialize");
    assert_eq!(init["id"], 1);
    let note: Value = serde_json::from_str(&initialized_body()).unwrap_or_default();
    assert_eq!(note["method"], "notifications/initialized");
    assert!(note.get("id").is_none());
}

#[test]
fn action_bodies_name_the_tool_and_arguments() {
    let (tool, args) = action_call(RoutineAction::Trigger, "r1", 0);
    assert_eq!(tool, "trigger_routine");
    assert_eq!(args, json!({"id": "r1"}));
    let body: Value = serde_json::from_str(&tool_call_body(2, tool, &args)).unwrap_or_default();
    assert_eq!(body["method"], "tools/call");
    assert_eq!(body["params"]["name"], "trigger_routine");
    assert_eq!(body["params"]["arguments"]["id"], "r1");

    let (tool, args) = action_call(RoutineAction::SetEnabled(false), "r1", 0);
    assert_eq!(tool, "update_routine");
    assert_eq!(args, json!({"id": "r1", "enabled": false}));
    let (tool, args) = action_call(RoutineAction::SetEnabled(true), "r1", 0);
    assert_eq!(
        (tool, args),
        ("update_routine", json!({"id": "r1", "enabled": true}))
    );

    let (tool, args) = action_call(RoutineAction::Snooze, "r1", 1_000);
    assert_eq!(tool, "snooze_routine");
    assert_eq!(
        args,
        json!({"id": "r1", "snoozed_until": 1_000 + SNOOZE_SECS})
    );
}
