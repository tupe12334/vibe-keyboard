/// Where a key's image comes from, read from the stored text.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ImageKind {
    None,
    File,
    Icon,
    App,
}

impl ImageKind {
    pub(super) const ALL: [Self; 4] = [Self::None, Self::File, Self::Icon, Self::App];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::File => "File",
            Self::Icon => "Icon",
            Self::App => "App icon",
        }
    }

    pub(super) fn of(image: Option<&std::path::Path>) -> Self {
        let Some(path) = image else {
            return Self::None;
        };
        let text = path.to_string_lossy();
        if text.starts_with("icon:") {
            Self::Icon
        } else if text.starts_with("app:") {
            Self::App
        } else {
            Self::File
        }
    }
}
