#![allow(clippy::expect_used, reason = "test: setup failures should panic")]

use super::*;

#[test]
fn share_links_are_accepted_and_other_links_refused() {
    assert!(is_share_link("https://photos.app.goo.gl/JEadX4KBqViVFXdm9"));
    assert!(is_share_link(
        " https://photos.google.com/share/AF1Qip?key=abc \n"
    ));
    assert!(!is_share_link("https://photos.google.com/photo/AF1Qip"));
    assert!(!is_share_link("http://photos.app.goo.gl/x"));
    assert!(!is_share_link("https://example.com/photos.app.goo.gl/x"));
}

#[test]
fn cover_photo_is_cropped_square() {
    let html = r#"<meta property="og:image" content="https://lh3.googleusercontent.com/pw/AP1Gcz=w600-h315-p-k">"#;
    assert_eq!(
        cover_photo_url(html).as_deref(),
        Some("https://lh3.googleusercontent.com/pw/AP1Gcz=w512-h512-c")
    );
}

#[test]
fn cover_photo_without_size_options_still_gets_the_crop() {
    let html = r#"property="og:image" content="https://lh3.googleusercontent.com/pw/X""#;
    assert_eq!(
        cover_photo_url(html).as_deref(),
        Some("https://lh3.googleusercontent.com/pw/X=w512-h512-c")
    );
}

#[test]
fn pages_without_a_google_photo_give_none() {
    assert_eq!(cover_photo_url("<html></html>"), None);
    assert_eq!(
        cover_photo_url(r#"property="og:image" content="unterminated"#),
        None
    );
    let other = r#"property="og:image" content="https://evil.example/x.jpg""#;
    assert_eq!(cover_photo_url(other), None);
}

/// Real request to a public sample album. Run with `cargo test -- --ignored google_photos`.
/// Set `GOOGLE_PHOTOS_LINK` to try another share link.
#[test]
#[ignore = "network"]
fn a_public_album_downloads_into_images() {
    let root = std::env::temp_dir().join(format!("vibe-keyboard-gphotos-{}", std::process::id()));
    let link = std::env::var("GOOGLE_PHOTOS_LINK")
        .unwrap_or_else(|_| "https://photos.app.goo.gl/JEadX4KBqViVFXdm9".into());
    let imported = import_shared_photo(&root, &link);
    let path = imported.expect("import");
    assert!(path.starts_with("images"), "{}", path.display());
    let photo = image::open(root.join(&path)).expect("decode");
    assert_eq!((photo.width(), photo.height()), (512, 512));
    let _ = std::fs::remove_dir_all(&root);
}
