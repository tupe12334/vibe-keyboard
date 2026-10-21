use crate::domain::profile::{inner_rect, PixelRect, SideInset};

/// Side screen render size, portrait: the panel is a vertical column. The upload rotates
/// it 90° counter-clockwise (`usb::side_upload`). Never render taller than this.
pub const SIDE_WIDTH: u32 = 100;
pub const SIDE_HEIGHT: u32 = 480;
/// Gap between the widget stack and the edges of the visible rect.
const MARGIN: u32 = 4;

/// The visible part of the column after the side insets. Content goes inside it.
pub fn safe_rect(inset: &SideInset) -> PixelRect {
    inner_rect(SIDE_WIDTH, SIDE_HEIGHT, inset.top, inset.bottom, 0, 0)
}

/// Widget stack: the visible rect less `MARGIN` on each side.
pub(super) const fn stack_rect(safe: PixelRect) -> PixelRect {
    PixelRect {
        x: safe.x + MARGIN,
        y: safe.y + MARGIN,
        w: safe.w.saturating_sub(2 * MARGIN),
        h: safe.h.saturating_sub(2 * MARGIN),
    }
}
