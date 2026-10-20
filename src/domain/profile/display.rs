use serde::{Deserialize, Serialize};

/// Bands the bezel hides. Content goes inside them; the background fills them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct DisplaySettings {
    /// Per-key margins, in px of the 85x85 key canvas.
    #[serde(default, skip_serializing_if = "is_default")]
    pub key_inset: KeyInset,
    /// Side screen margins, in px of the 100x480 portrait canvas.
    #[serde(default, skip_serializing_if = "is_default")]
    pub side_inset: SideInset,
}

/// Key margins. The default hides the top 8 px of each key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct KeyInset {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

impl Default for KeyInset {
    fn default() -> Self {
        Self {
            top: 8,
            bottom: 0,
            left: 0,
            right: 0,
        }
    }
}

/// Side screen margins. The defaults keep the old safe area, y 30..440.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct SideInset {
    pub top: u32,
    pub bottom: u32,
}

impl Default for SideInset {
    fn default() -> Self {
        Self {
            top: 30,
            bottom: 40,
        }
    }
}

/// Skips a field in TOML when it holds the default value, so old files stay byte-stable.
pub fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// A pixel rectangle inside a canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// The part of a `width` x `height` canvas that the insets leave visible. Insets larger than
/// the canvas leave an empty rect, never a panic.
pub fn inner_rect(
    width: u32,
    height: u32,
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
) -> PixelRect {
    PixelRect {
        x: left.min(width),
        y: top.min(height),
        w: width.saturating_sub(left).saturating_sub(right),
        h: height.saturating_sub(top).saturating_sub(bottom),
    }
}

#[cfg(test)]
#[path = "display_tests.rs"]
#[allow(
    clippy::expect_used,
    reason = "test module: setup failures should panic"
)]
mod display_tests;
