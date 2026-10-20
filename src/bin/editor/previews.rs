use eframe::egui::{self, Color32, Rect, Vec2};
use vibe_keyboard::domain::profile::{KeyConfig, KeyInset, PixelRect, SideConfig, SideInset};
use vibe_keyboard::infrastructure::system_metrics::Metrics;

/// Upload an RGB image as an egui texture, fully opaque.
pub(super) fn texture_from(
    ctx: &egui::Context,
    name: &str,
    img: &image::RgbImage,
) -> egui::TextureHandle {
    let size = [img.width() as usize, img.height() as usize];
    let rgba: Vec<u8> = img
        .pixels()
        .flat_map(|p| [p.0[0], p.0[1], p.0[2], 255])
        .collect();
    let color = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    ctx.load_texture(name.to_string(), color, egui::TextureOptions::LINEAR)
}

/// A rendered key preview.
pub(super) struct Preview {
    pub(super) config: Option<KeyConfig>,
    pub(super) inset: KeyInset,
    /// `app_icon_generation()` when rendered, so finished app lookups show up.
    pub(super) generation: u64,
    pub(super) texture: egui::TextureHandle,
}

/// Rendered side preview, with the config and insets it was rendered from.
pub(super) struct SidePreview {
    pub(super) config: Option<SideConfig>,
    pub(super) inset: SideInset,
    pub(super) metrics: Metrics,
    pub(super) texture: egui::TextureHandle,
}

/// Shades the bands the bezel hides. `area` shows a canvas of `canvas` pixels, and `visible`
/// is the part left after the insets, in the same pixels.
pub(super) fn shade_hidden(
    painter: &egui::Painter,
    area: Rect,
    canvas: (u32, u32),
    visible: PixelRect,
) {
    let (width, height) = canvas;
    let scale = Vec2::new(area.width() / width as f32, area.height() / height as f32);
    let bottom = visible.y + visible.h;
    let right = visible.x + visible.w;
    let bands = [
        (0, 0, width, visible.y),
        (0, bottom, width, height.saturating_sub(bottom)),
        (0, visible.y, visible.x, visible.h),
        (right, visible.y, width.saturating_sub(right), visible.h),
    ];
    for (x, y, band_w, band_h) in bands {
        if band_w == 0 || band_h == 0 {
            continue;
        }
        let min = area.min + Vec2::new(x as f32 * scale.x, y as f32 * scale.y);
        let size = Vec2::new(band_w as f32 * scale.x, band_h as f32 * scale.y);
        painter.rect_filled(
            Rect::from_min_size(min, size),
            0.0,
            Color32::from_rgba_unmultiplied(230, 60, 60, 110),
        );
    }
}
