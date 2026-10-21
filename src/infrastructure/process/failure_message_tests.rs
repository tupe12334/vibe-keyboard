use super::failure_message;

#[test]
fn includes_code_and_first_stderr_line() {
    assert_eq!(
        failure_message("osascript", Some(1), "  execution error: boom  \nsecond\n"),
        "osascript exited with code 1: execution error: boom"
    );
}

#[test]
fn skips_blank_lines_before_the_first_text() {
    assert_eq!(
        failure_message("ps", Some(3), "\n  \nreal\n"),
        "ps exited with code 3: real"
    );
}

#[test]
fn empty_stderr_gives_code_only() {
    assert_eq!(failure_message("ps", Some(3), ""), "ps exited with code 3");
    assert_eq!(
        failure_message("ps", Some(3), "  \n\n"),
        "ps exited with code 3"
    );
}

#[test]
fn missing_code_prints_none() {
    assert_eq!(failure_message("ps", None, ""), "ps exited with code none");
}

#[test]
fn long_stderr_line_is_cut_to_200_chars() {
    let long = "x".repeat(300);
    assert_eq!(
        failure_message("ps", Some(1), &long),
        format!("ps exited with code 1: {}", "x".repeat(200))
    );
}

#[test]
fn truncation_counts_chars_not_bytes() {
    let msg = failure_message("ps", Some(1), &"é".repeat(300));
    let line = msg.rsplit(": ").next().unwrap_or_default();
    assert_eq!(line.chars().count(), 200);
}
