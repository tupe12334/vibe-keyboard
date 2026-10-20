use std::path::Path;

use super::super::summary_from_jsonl;

/// The session name from the newest transcript for `cwd` under `~/.claude/projects`. Claude Code
/// stores transcripts in a folder named after the path with every non-alphanumeric as `-`.
pub(super) fn summary_for(cwd: &str, home: &str) -> Option<String> {
    let encoded: String = cwd
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let dir = Path::new(home).join(".claude/projects").join(encoded);
    let newest = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
        .max_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok())?;
    summary_from_jsonl(&std::fs::read_to_string(newest.path()).ok()?)
}
