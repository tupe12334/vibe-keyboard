//! Side screen preview for the editor, drawn by the same `side_image` the deck uses.

use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::images::side_image;
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::infrastructure::system_metrics::{Disk, Memory, Metrics};

use crate::key_commands::png_data_url;

/// PNG data URL of the side screen of `page`, drawn by the same `side_image` the deck uses.
/// The values are a fixed sample, so the preview reads the same on every machine.
#[tauri::command]
#[specta::specta]
pub fn side_preview(profile: Profile, page: u32) -> Result<String, String> {
    let side = profile
        .pages
        .get(page as usize)
        .and_then(|p| p.side.as_ref());
    let image = side_image(
        side,
        &profile_store::root_dir(),
        &sample_metrics(),
        &profile.display.side_inset,
    );
    png_data_url(&image)
}

fn sample_metrics() -> Metrics {
    Metrics {
        cpu_percent: 62,
        disk: Some(Disk {
            free: 412_000_000_000,
            total: 994_000_000_000,
        }),
        temperature: Some(58.0),
        memory: Some(Memory {
            used: 9_000_000_000,
            total: 16_000_000_000,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_preview_is_a_png_for_a_page_with_and_without_a_side_screen() {
        let mut profile = Profile::default_from_legacy();
        let bare = side_preview(profile.clone(), 0);
        assert!(bare.is_ok_and(|url| url.starts_with("data:image/png;base64,")));
        profile.set_side(
            0,
            Some(vibe_keyboard::domain::profile::SideConfig::default()),
        );
        assert!(side_preview(profile.clone(), 0).is_ok());
        assert!(
            side_preview(profile, 9).is_ok(),
            "a missing page shows a black column"
        );
    }
}
