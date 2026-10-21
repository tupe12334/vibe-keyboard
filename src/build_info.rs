/// Line that names the exact build. Format: `vibe-keyboard <version> (<sha>, built <date>)`.
#[must_use]
pub fn format_build_line(version: &str, sha: &str, date: &str) -> String {
    format!("vibe-keyboard {version} ({sha}, built {date})")
}

/// Build line for this binary. Written as the first `app.log` line at startup.
#[must_use]
pub fn startup_line() -> String {
    format_build_line(
        env!("CARGO_PKG_VERSION"),
        env!("VK_GIT_SHA"),
        env!("VK_BUILD_DATE"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_version_sha_and_date() {
        assert_eq!(
            format_build_line("0.1.1", "abc1234-dirty", "2026-10-21"),
            "vibe-keyboard 0.1.1 (abc1234-dirty, built 2026-10-21)"
        );
    }

    #[test]
    fn startup_line_names_package_version() {
        let line = startup_line();
        assert!(line.starts_with(concat!("vibe-keyboard ", env!("CARGO_PKG_VERSION"), " (")));
        assert!(line.ends_with(')'));
    }
}
