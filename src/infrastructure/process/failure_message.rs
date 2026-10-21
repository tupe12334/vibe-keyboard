//! Formats the WARN line for a child process that failed. Pure, so the coverage gate measures it.

/// Longest stderr line kept in the message.
const MAX_LINE_CHARS: usize = 200;

/// `"<program> exited with code <code>"`, plus `": <first non-empty stderr line>"` when there is one.
pub fn failure_message(program: &str, code: Option<i32>, stderr: &str) -> String {
    let code = code.map_or_else(|| "none".to_string(), |c| c.to_string());
    match stderr.lines().map(str::trim).find(|line| !line.is_empty()) {
        Some(line) => {
            let line: String = line.chars().take(MAX_LINE_CHARS).collect();
            format!("{program} exited with code {code}: {line}")
        }
        None => format!("{program} exited with code {code}"),
    }
}

#[cfg(test)]
#[path = "failure_message_tests.rs"]
mod tests;
