//! Key images from a Google Photos share link. No sign-in and no API key: a shared album or
//! photo page is public, and its `og:image` tag names the cover photo. The download lives in
//! `google_photos/os.rs`.

mod os;
pub use os::import_shared_photo;

/// Square size the photo server crops to. `key_image` scales it down to the key.
const SIZE: u32 = 512;

const SHARE_PREFIXES: [&str; 2] = [
    "https://photos.app.goo.gl/",
    "https://photos.google.com/share/",
];

const OG_IMAGE: &str = r#"property="og:image" content=""#;

const PHOTO_HOST: &str = "https://lh3.googleusercontent.com/";

/// Whether `link` is a Google Photos share link. Other links are refused before any request.
pub fn is_share_link(link: &str) -> bool {
    let link = link.trim();
    SHARE_PREFIXES.iter().any(|prefix| link.starts_with(prefix))
}

/// The cover photo of a share page, as a URL for a square crop. `None` when the page has none.
/// For an album the cover is its first photo; for a single shared photo it is that photo.
pub fn cover_photo_url(html: &str) -> Option<String> {
    let start = html.find(OG_IMAGE)? + OG_IMAGE.len();
    let rest = &html[start..];
    let url = &rest[..rest.find('"')?];
    // ponytail: `=w600-h315-p-k` style size options follow the first `=`; swap in our own.
    let base = url.split('=').next().unwrap_or(url);
    base.starts_with(PHOTO_HOST)
        .then(|| format!("{base}=w{SIZE}-h{SIZE}-c"))
}

#[cfg(test)]
#[path = "google_photos_tests.rs"]
mod tests;
