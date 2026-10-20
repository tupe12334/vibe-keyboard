use std::collections::HashSet;

use super::*;

#[path = "plugins_tests/fake_host.rs"]
mod fake_host;
#[path = "plugins_tests/runs.rs"]
mod runs;

#[test]
fn names_are_unique_non_empty_and_found() {
    let mut seen = HashSet::new();
    for name in names() {
        assert!(!name.is_empty());
        assert!(seen.insert(name), "duplicate plugin '{name}'");
        assert_eq!(find(name).map(Plugin::name), Some(name));
    }
    assert!(find("no_such_plugin").is_none());
}

/// Saved `profile.toml` files store these ids, and the first one is the editor default.
#[test]
fn plugin_ids_are_pinned_in_order() {
    let ids: Vec<_> = names().collect();
    assert_eq!(
        ids,
        [
            "centy_projects",
            "terminal",
            "claude",
            "centy_web",
            "numpad",
            "log_file",
            "vscode_config",
            "claude_sessions",
            "moadim",
        ]
    );
}

#[test]
fn every_plugin_has_a_label_and_all_but_claude_sessions_a_tile() {
    for plugin in PLUGINS {
        assert!(!plugin.label().is_empty(), "{} has no label", plugin.name());
        let has_tile = plugin.tile().is_some();
        assert_eq!(
            has_tile,
            plugin.name() != "claude_sessions",
            "{}",
            plugin.name()
        );
    }
}
