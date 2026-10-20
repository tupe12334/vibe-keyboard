use std::collections::HashMap;

use crate::domain::actions::Glyph;

use super::session_status::is_spinner;
use super::{is_valid_tty, CLAUDE_SUFFIX, SUMMARY_CHARS};

/// Glyph and title from a terminal tab title such as `✳ Centy availability (claude)`.
/// The title keeps no glyph and no ` (claude)` suffix.
pub fn parse_title(raw: &str) -> (Option<Glyph>, String) {
    let raw = raw.trim();
    let (glyph, rest) = match raw.chars().next() {
        Some('✳') => (Some(Glyph::Waiting), &raw['✳'.len_utf8()..]),
        Some(c) if is_spinner(c) => (Some(Glyph::Spinner), &raw[c.len_utf8()..]),
        _ => (None, raw),
    };
    let rest = rest.trim();
    let title = rest.strip_suffix(CLAUDE_SUFFIX).unwrap_or(rest).trim_end();
    (glyph, title.to_string())
}

/// Tab titles keyed by bare tty, from `<device>\t<title>` lines. Lines with a bad tty are dropped.
pub fn parse_tab_titles(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let (device, title) = line.split_once('\t')?;
            let tty = device.strip_prefix("/dev/")?;
            is_valid_tty(tty).then(|| (tty.to_string(), title.to_string()))
        })
        .collect()
}

/// The session name in a Claude Code transcript (`.jsonl`). Uses the last `summary` line, else
/// the first user prompt. The result is cut to `SUMMARY_CHARS`.
pub fn summary_from_jsonl(text: &str) -> Option<String> {
    let mut summary: Option<String> = None;
    let mut first_prompt: Option<String> = None;
    for value in text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
    {
        match value.get("type").and_then(serde_json::Value::as_str) {
            Some("summary") => {
                summary = value
                    .get("summary")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .or(summary);
            }
            // A prompt that is only a pasted link makes a poor session name, so skip it.
            Some("user") if first_prompt.is_none() => {
                first_prompt = user_text(&value).filter(|text| !is_url(text));
            }
            _ => {}
        }
    }
    summary
        .or(first_prompt)
        .map(|text| text.chars().take(SUMMARY_CHARS).collect())
}

fn is_url(text: &str) -> bool {
    let text = text.trim_start();
    text.starts_with("http://") || text.starts_with("https://")
}

/// Prompt text of a user line. Content is either a string or a list of parts with `text`.
fn user_text(value: &serde_json::Value) -> Option<String> {
    match value.pointer("/message/content")? {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(parts) => parts
            .iter()
            .find_map(|part| part.get("text").and_then(serde_json::Value::as_str))
            .map(str::to_string),
        _ => None,
    }
}
