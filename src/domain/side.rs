use super::profile::SideConfig;

/// What the side screen shows after a redraw.
#[derive(Debug, PartialEq)]
pub enum SideContent {
    /// Leave the side screen as it is.
    Keep,
    /// Show this side screen.
    Show(SideConfig),
}

/// The one decision for the side screen. A screen's own declaration wins. Otherwise the
/// side of the home page stays. With neither, the side screen is kept.
pub fn resolve_side(
    main_page_side: Option<SideConfig>,
    screen_side: Option<SideConfig>,
) -> SideContent {
    match screen_side.or(main_page_side) {
        Some(side) => SideContent::Show(side),
        None => SideContent::Keep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn side(label: &str) -> SideConfig {
        SideConfig {
            label: label.into(),
            ..SideConfig::default()
        }
    }

    #[test]
    fn built_in_without_declaration_keeps_page_side() {
        assert_eq!(
            resolve_side(Some(side("page")), None),
            SideContent::Show(side("page"))
        );
    }

    #[test]
    fn declaration_overrides_page_side() {
        assert_eq!(
            resolve_side(Some(side("page")), Some(side("screen"))),
            SideContent::Show(side("screen"))
        );
    }

    #[test]
    fn declaration_shows_without_page_side() {
        assert_eq!(
            resolve_side(None, Some(side("screen"))),
            SideContent::Show(side("screen"))
        );
    }

    #[test]
    fn no_page_side_and_no_declaration_keeps_side_screen() {
        assert_eq!(resolve_side(None, None), SideContent::Keep);
    }
}
