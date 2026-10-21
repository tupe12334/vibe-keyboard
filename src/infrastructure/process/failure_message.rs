//! Formats the log line for a child process that failed, and emits it at the chosen level. The
//! formatting is pure, so the coverage gate measures it.

/// Longest stderr line kept in the message.
const MAX_LINE_CHARS: usize = 200;

/// How a failed child is logged. Each call site picks one by how often it fails and what its
/// args hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logging {
    /// WARN with the first stderr line. For actions the user triggers.
    Warn,
    /// WARN with the exit code only. For calls whose args carry user text, which an error can echo.
    WarnCodeOnly,
    /// DEBUG with the first stderr line. For routine probes that fail often, such as refresh polls.
    Debug,
}

pub fn emit(logging: Logging, message: &str) {
    match logging {
        Logging::Debug => tracing::debug!("{}", message),
        Logging::Warn | Logging::WarnCodeOnly => tracing::warn!("{}", message),
    }
}

/// The message for a non-zero exit. `WarnCodeOnly` drops stderr, so the text stays out of the log.
pub fn exit_message(program: &str, code: Option<i32>, stderr: &str, logging: Logging) -> String {
    let stderr = if logging == Logging::WarnCodeOnly {
        ""
    } else {
        stderr
    };
    failure_message(program, code, stderr)
}

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
