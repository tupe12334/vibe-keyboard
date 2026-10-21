/// Why a hold ended. Written to the key-up log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    /// The user let go of the key.
    Key,
    /// Focus left the hold's app for two probes in a row. `to` is the new app, if known.
    Moved { to: Option<i32> },
    /// Accessibility is off.
    Untrusted,
    /// The app did not touch the guard for the watchdog time.
    Watchdog,
    /// The screen changed.
    Screen,
    /// The device disconnected.
    Disconnected,
    /// The app is stopping.
    Shutdown,
}

impl Release {
    /// The reason as the key-up log shows it.
    pub fn label(self) -> String {
        match self {
            Self::Key => "released".into(),
            Self::Moved { to: Some(pid) } => format!("moved to pid {pid}"),
            Self::Moved { to: None } => "moved, no frontmost app".into(),
            Self::Untrusted => "untrusted".into(),
            Self::Watchdog => "watchdog".into(),
            Self::Screen => "screen change".into(),
            Self::Disconnected => "device lost".into(),
            Self::Shutdown => "shutdown".into(),
        }
    }
}
