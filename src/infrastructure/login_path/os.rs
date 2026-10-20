//! Reads `PATH` and `HOME` and sets `PATH` for this process. Excluded from the coverage gate.

use std::env;
use std::path::Path;

use super::merge_path;

/// Extends `PATH` for this process and returns the directories it appended.
///
/// Call it first thing in `main`, before any thread starts. Why it is sound there: `env::set_var`
/// is only unsafe to call while another thread may read the environment. No thread exists yet at
/// that point, so nothing can race with the write. The crate uses edition 2021, where `set_var`
/// is a safe call.
pub fn apply_login_path() -> Vec<String> {
    let current = env::var("PATH").unwrap_or_default();
    let home = env::var("HOME").unwrap_or_default();
    let merged = merge_path(&current, &home, Path::exists);
    env::set_var("PATH", &merged);
    merged
        .split(':')
        .filter(|dir| !current.split(':').any(|kept| kept == *dir))
        .map(str::to_owned)
        .collect()
}
