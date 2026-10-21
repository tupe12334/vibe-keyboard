use super::*;

#[test]
fn failing_command_message_has_code_and_stderr() {
    let Some(out) = capture(
        "sh",
        &["-c", "echo boom >&2; exit 3"],
        Duration::from_secs(5),
    ) else {
        panic!("sh did not run");
    };
    assert!(!out.status.success());
    let msg = failure_message("sh", out.status.code(), &out.stderr);
    assert!(msg.contains('3') && msg.contains("boom"), "{msg}");
}
