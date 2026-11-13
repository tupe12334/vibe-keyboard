//! Font data bundled into the binaries.

/// Noto Sans Hebrew Regular. Covers Hebrew and Latin. License: SIL OFL 1.1 (`assets/fonts/OFL.txt`).
pub const NOTO_SANS_HEBREW: &[u8] = include_bytes!("../assets/fonts/NotoSansHebrew-Regular.ttf");

/// The egui default fonts, plus `NOTO_SANS_HEBREW` as the last fallback in both families.
pub fn egui_fonts() -> eframe::egui::FontDefinitions {
    use eframe::egui::{FontData, FontDefinitions, FontFamily};
    const NAME: &str = "noto_sans_hebrew";
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        NAME.to_owned(),
        std::sync::Arc::new(FontData::from_static(NOTO_SANS_HEBREW)),
    );
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push(NAME.to_owned());
    }
    fonts
}

#[cfg(test)]
mod tests {
    use eframe::egui::FontFamily;

    #[test]
    fn egui_fonts_adds_hebrew_as_last_fallback() {
        let fonts = super::egui_fonts();
        assert!(fonts.font_data.contains_key("noto_sans_hebrew"));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            let names = &fonts.families[&family];
            assert_eq!(names.last().map(String::as_str), Some("noto_sans_hebrew"));
        }
    }
}
