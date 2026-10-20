use super::{Hotkey, Key, Modifier};

impl Hotkey {
    /// `AppleScript` that sends this shortcut via System Events.
    pub fn applescript(&self) -> String {
        let send = match &self.key {
            Key::Char(c) => format!("keystroke \"{}\"", escape_applescript(&c.to_string())),
            Key::Code(code) => format!("key code {code}"),
        };
        if self.modifiers.is_empty() {
            return format!("tell application \"System Events\" to {send}");
        }
        let mods: Vec<&str> = self
            .modifiers
            .iter()
            .map(|m| match m {
                Modifier::Command => "command down",
                Modifier::Shift => "shift down",
                Modifier::Option => "option down",
                Modifier::Control => "control down",
            })
            .collect();
        let mods = mods.join(", ");
        format!("tell application \"System Events\" to {send} using {{{mods}}}")
    }
}

fn escape_applescript(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}
