//! Writes the icons of a few installed apps to `/tmp` to check `app_icon`.
//!
//! Run: `cargo run --example app_icon`

// The app is a bin-only crate, so the module is included by path.
#[path = "../src/infrastructure/app_icon.rs"]
mod app_icon;

const APPS: [(&str, &str); 3] = [
    ("com.google.Chrome", "/tmp/app-icon-chrome.png"),
    ("com.googlecode.iterm2", "/tmp/app-icon-iterm.png"),
    ("com.apple.finder", "/tmp/app-icon-finder.png"),
];

fn main() {
    for (bundle_id, out) in APPS {
        match app_icon::app_icon(bundle_id, 85) {
            Some(img) => match img.save(out) {
                Ok(()) => eprintln!("{bundle_id}: {}x{} -> {out}", img.width(), img.height()),
                Err(e) => eprintln!("{bundle_id}: save failed: {e}"),
            },
            None => eprintln!("{bundle_id}: no icon found"),
        }
    }
}
