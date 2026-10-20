use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use image::imageops::FilterType;
use image::RgbImage;

use crate::infrastructure::app_icon::app_icon;

/// Installed-app icons, looked up off-thread. Results are cached, misses included.
#[derive(Default)]
struct AppIconCache {
    /// Finished lookups by (bundle id, size). `None` is a miss (no such app, or no icon).
    ready: HashMap<(String, u32), Option<RgbImage>>,
    /// Lookups still running on a worker thread.
    pending: HashSet<(String, u32)>,
}

static APP_ICONS: OnceLock<Mutex<AppIconCache>> = OnceLock::new();
/// Bumped whenever a lookup finishes, so callers know to re-render.
static APP_ICON_GENERATION: AtomicU64 = AtomicU64::new(0);

fn app_cache() -> std::sync::MutexGuard<'static, AppIconCache> {
    APP_ICONS
        .get_or_init(|| Mutex::new(AppIconCache::default()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Counter that changes each time an app icon lookup finishes.
pub fn app_icon_generation() -> u64 {
    APP_ICON_GENERATION.load(Ordering::Relaxed)
}

/// The icon for `bundle_id` if it is already known. Otherwise start a background lookup and
/// return `None`, so the key shows its label until the lookup finishes.
pub(super) fn app_icon_cached(bundle_id: &str, size: u32) -> Option<RgbImage> {
    let key = (bundle_id.to_string(), size);
    let mut cache = app_cache();
    if let Some(known) = cache.ready.get(&key) {
        return known.clone();
    }
    if cache.pending.insert(key.clone()) {
        std::thread::spawn(move || {
            let icon = app_icon(&key.0, key.1).map(|img| {
                img.resize_to_fill(key.1, key.1, FilterType::Triangle)
                    .to_rgb8()
            });
            let mut cache = app_cache();
            cache.pending.remove(&key);
            cache.ready.insert(key, icon);
            APP_ICON_GENERATION.fetch_add(1, Ordering::Relaxed);
        });
    }
    None
}
