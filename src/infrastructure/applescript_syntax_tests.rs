//! Compiles every `AppleScript` the app builds, with `osacompile`. Syntax only: `osacompile` never
//! runs a script, so no terminal opens and no key is sent.

use std::path::Path;
use std::process::Command;

#[path = "applescript_syntax_tests/scripts.rs"]
mod scripts;

/// iTerm2 scripts compile only against its scripting dictionary, which CI runners lack. They are
/// checked only where iTerm2 is installed.
fn iterm_installed() -> bool {
    Path::new("/Applications/iTerm.app").exists()
}

#[test]
fn every_applescript_compiles_with_osacompile() {
    let dir = std::env::temp_dir().join(format!("vibe-keyboard-osacompile-{}", std::process::id()));
    if let Err(err) = std::fs::create_dir_all(&dir) {
        panic!("cannot create {}: {err}", dir.display());
    }
    let mut all = scripts::scripts();
    assert_eq!(
        all.len(),
        scripts::SCRIPT_COUNT,
        "a builder returned no script"
    );
    if std::env::var_os("CI").is_some() {
        assert!(
            iterm_installed(),
            "CI has no /Applications/iTerm.app, so iTerm2 scripts would be skipped"
        );
    }
    all.retain(|(_, text)| iterm_installed() || !text.contains("iTerm2"));
    let mut failures = Vec::new();
    for (index, (label, text)) in all.iter().enumerate() {
        let out = dir.join(format!("script-{index}.scpt"));
        match Command::new("osacompile")
            .arg("-o")
            .arg(&out)
            .arg("-e")
            .arg(text)
            .output()
        {
            Ok(result) if result.status.success() => {}
            Ok(result) => failures.push(format!(
                "{label}: {}",
                String::from_utf8_lossy(&result.stderr).trim()
            )),
            Err(err) => failures.push(format!("{label}: osacompile did not start: {err}")),
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        failures.is_empty(),
        "AppleScript did not compile:\n{}",
        failures.join("\n")
    );
}
