use crate::domain::profile::{Action, BUILTIN_NAMES};

/// The kind of an `Action`, without its fields. Used by the editor's type picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    None,
    OpenUrl,
    RunCommand,
    OpenApp,
    Hotkey,
    GoToPage,
    Builtin,
    PushToTalk,
}

impl ActionKind {
    pub const ALL: [Self; 8] = [
        Self::None,
        Self::OpenUrl,
        Self::RunCommand,
        Self::OpenApp,
        Self::Hotkey,
        Self::GoToPage,
        Self::Builtin,
        Self::PushToTalk,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::OpenUrl => "Open URL",
            Self::RunCommand => "Run command",
            Self::OpenApp => "Open app",
            Self::Hotkey => "Hotkey",
            Self::GoToPage => "Go to page",
            Self::Builtin => "Built-in",
            Self::PushToTalk => "Push to talk",
        }
    }
}

impl Action {
    pub const fn kind(&self) -> ActionKind {
        match self {
            Self::None => ActionKind::None,
            Self::OpenUrl { .. } => ActionKind::OpenUrl,
            Self::RunCommand { .. } => ActionKind::RunCommand,
            Self::OpenApp { .. } => ActionKind::OpenApp,
            Self::Hotkey { .. } => ActionKind::Hotkey,
            Self::GoToPage { .. } => ActionKind::GoToPage,
            Self::Builtin { .. } => ActionKind::Builtin,
            Self::PushToTalk { .. } => ActionKind::PushToTalk,
        }
    }

    /// A fresh action of `kind` with empty or first-choice fields.
    pub fn default_of(kind: ActionKind) -> Self {
        match kind {
            ActionKind::None => Self::None,
            ActionKind::OpenUrl => Self::OpenUrl { url: String::new() },
            ActionKind::RunCommand => Self::RunCommand {
                command: String::new(),
            },
            ActionKind::OpenApp => Self::OpenApp {
                name: String::new(),
            },
            ActionKind::Hotkey => Self::Hotkey {
                keys: String::new(),
            },
            ActionKind::GoToPage => Self::GoToPage { page: 0 },
            ActionKind::PushToTalk => Self::PushToTalk { key: None },
            ActionKind::Builtin => Self::Builtin {
                name: BUILTIN_NAMES[0].into(),
            },
        }
    }
}
