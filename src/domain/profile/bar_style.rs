use serde::{Deserialize, Serialize};

use super::parse_color;

/// Narrowest and widest CPU bar the editors accept, in px of the 100 px wide column.
pub const BAR_THICKNESS_MIN: u32 = 10;
pub const BAR_THICKNESS_MAX: u32 = 60;
/// Bar width when `thickness` is absent.
pub const BAR_THICKNESS_DEFAULT: u32 = 40;
/// Start colors for the editor when the user turns a custom color on.
pub const BAR_FILL_DEFAULT: &str = "#2EA043";
pub const BAR_TRACK_DEFAULT: &str = "#3C3C3C";

/// Look of the CPU bar on one side screen. Every field is optional. Absent means today's look,
/// so a profile without `bar_style` renders as before.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BarStyle {
    /// Fill color `#RRGGBB`. Absent: green below 60 %, amber below 85 %, red above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    /// Color of the unfilled part, `#RRGGBB`. Absent: dark grey.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    /// Bar width in px. Absent: 40.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thickness: Option<u32>,
    /// Show the percent under the bar. Absent: shown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_value: Option<bool>,
    /// Color of the percent text, `#RRGGBB`. Absent: the side text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_color: Option<String>,
}

impl BarStyle {
    /// Problems with the style, without a page prefix. Empty when valid.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        let colors = [
            ("fill", &self.fill),
            ("track", &self.track),
            ("label color", &self.label_color),
        ];
        for (name, color) in colors {
            if color.as_deref().is_some_and(|c| parse_color(c).is_none()) {
                out.push(format!("bar {name} must be #RRGGBB"));
            }
        }
        if self
            .thickness
            .is_some_and(|t| !(BAR_THICKNESS_MIN..=BAR_THICKNESS_MAX).contains(&t))
        {
            out.push(format!(
                "bar thickness must be {BAR_THICKNESS_MIN} to {BAR_THICKNESS_MAX} px"
            ));
        }
        out
    }
}
