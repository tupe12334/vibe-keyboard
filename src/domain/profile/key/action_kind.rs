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
