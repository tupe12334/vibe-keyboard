use crate::config::Action;
use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;
#[allow(unused_imports)]
use tracing::warn;

/// Execute an action, spawning subprocesses in the background.
/// This function is intentionally non-blocking — the child process runs
/// independently after spawn().
pub fn execute(action: &Action) -> Result<()> {
    match action {
        Action::Launch { command, args } => launch(command, args),
        Action::Script { command } => script(command),
        Action::Keypress { keys } => keypress(keys),
    }
}

fn launch(command: &str, args: &[String]) -> Result<()> {
    info!("Launch: {} {:?}", command, args);
    Command::new(command)
        .args(args)
        .spawn()
        .with_context(|| format!("Failed to launch '{command}'"))?;
    Ok(())
}

fn script(command: &str) -> Result<()> {
    info!("Script: {}", command);
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .with_context(|| format!("Failed to run script '{command}'"))?;
    Ok(())
}

fn keypress(keys: &str) -> Result<()> {
    info!("Keypress: {}", keys);

    #[cfg(target_os = "macos")]
    return keypress_macos(keys);

    #[cfg(target_os = "linux")]
    return keypress_linux(keys);

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warn!("Keypress is not supported on this platform");
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn keypress_macos(keys: &str) -> Result<()> {
    let (key, modifiers) = parse_keys(keys);

    let script = if modifiers.is_empty() {
        format!(r#"tell application "System Events" to keystroke "{key}""#)
    } else {
        let mod_str = modifiers
            .iter()
            .map(|m| format!("{m} down"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(r#"tell application "System Events" to keystroke "{key}" using {{{mod_str}}}"#)
    };

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn()
        .context("osascript failed — make sure Accessibility permissions are granted")?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn keypress_linux(keys: &str) -> Result<()> {
    // xdotool accepts "ctrl+shift+t" directly
    let xdotool_keys = keys
        .replace("cmd", "super")
        .replace("command", "super");
    Command::new("xdotool")
        .arg("key")
        .arg(&xdotool_keys)
        .spawn()
        .context("xdotool failed — install xdotool or check DISPLAY")?;
    Ok(())
}

/// Split `"ctrl+shift+t"` → `("t", ["control", "shift"])` for osascript.
#[allow(dead_code)]
fn parse_keys(keys: &str) -> (String, Vec<String>) {
    let mut modifiers = Vec::new();
    let mut key = String::new();

    for part in keys.split('+') {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers.push("control".into()),
            "cmd" | "command" | "meta" | "win" => modifiers.push("command".into()),
            "alt" | "option" => modifiers.push("option".into()),
            "shift" => modifiers.push("shift".into()),
            other => key = other.to_string(),
        }
    }

    (key, modifiers)
}
