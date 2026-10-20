use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::bar_style::BarStyle;
use super::color::{default_bg, default_fg};

/// The side screen: a 480x100 strip. Same image sources as keys, else the label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SideConfig {
    #[serde(default)]
    pub label: String,
    #[serde(default = "default_bg")]
    pub bg: String,
    #[serde(default = "default_fg")]
    pub fg: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<PathBuf>,
    /// Live values shown left to right. When set, the side screen shows these instead of
    /// `label` and `image`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widgets: Vec<Widget>,
    /// Look of the CPU bar widget. Absent means the default look.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bar_style: Option<BarStyle>,
}

/// One value on the side screen. TOML: `type = "cpu_percent"`, `type = "text"`, etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Widget {
    /// Global CPU use as a number.
    CpuPercent,
    /// Global CPU use as a bar from 0 to 100 %.
    CpuBar,
    /// Global memory use as a bar from 0 to 100 %.
    MemBar,
    /// CPU and memory in one bar: the left half is CPU use, the right half is memory use.
    CpuMemBar,
    /// Hottest CPU sensor in degrees Celsius, or `N/A`.
    Temperature,
    /// Free space on the user volume.
    DiskFree,
    /// Fixed text.
    Text {
        #[serde(default)]
        value: String,
    },
}

impl Default for SideConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            bg: default_bg(),
            fg: default_fg(),
            image: None,
            widgets: Vec::new(),
            bar_style: None,
        }
    }
}
