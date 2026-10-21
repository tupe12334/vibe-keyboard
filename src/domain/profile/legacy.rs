use std::collections::BTreeMap;

use super::{Action, ClaudeSessionsConfig, DisplaySettings, InputConfig, KeyConfig, Page, Profile};

impl Profile {
    /// The layout that matches the hard-coded main pages before profiles existed.
    pub fn default_from_legacy() -> Self {
        let builtin = |label: &str, name: &str| KeyConfig {
            label: label.into(),
            action: Action::Builtin { name: name.into() },
            ..KeyConfig::default()
        };
        Self {
            prompts: Vec::new(),
            input: InputConfig::default(),
            claude_sessions: ClaudeSessionsConfig::default(),
            pages: vec![
                Page {
                    name: "Main".into(),
                    apps: Vec::new(),
                    side: None,
                    keys: BTreeMap::from([
                        (1, builtin("List Projects", "centy_projects")),
                        (2, builtin("Open Terminal", "terminal")),
                        (3, builtin("Open Claude", "claude")),
                        (4, builtin("Open Centy", "centy_web")),
                    ]),
                },
                Page {
                    name: "Tools".into(),
                    apps: Vec::new(),
                    side: None,
                    keys: BTreeMap::from([
                        (1, builtin("Input Number", "numpad")),
                        (14, builtin("Open Log", "log_file")),
                        (15, builtin("Open Config", "vscode_config")),
                    ]),
                },
            ],
            display: DisplaySettings::default(),
        }
    }
}
