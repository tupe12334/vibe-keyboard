use std::path::Path;

use super::*;

fn all_exist(_: &Path) -> bool {
    true
}

fn none_exist(_: &Path) -> bool {
    false
}

#[test]
fn appends_existing_dirs_in_order_after_current() {
    let merged = merge_path("/usr/bin:/bin", "/Users/me", all_exist);
    assert_eq!(
        merged,
        "/usr/bin:/bin:/Users/me/.local/bin:/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:\
         /Users/me/.nix-profile/bin:/nix/var/nix/profiles/default/bin:/Users/me/.volta/bin:\
         /Users/me/.cargo/bin"
    );
}

#[test]
fn adds_nothing_when_no_extra_dir_exists() {
    assert_eq!(
        merge_path("/usr/bin:/bin", "/Users/me", none_exist),
        "/usr/bin:/bin"
    );
}

#[test]
fn keeps_current_entries_first_and_in_order() {
    let merged = merge_path("/zzz:/usr/bin", "/Users/me", |p| {
        p == Path::new("/opt/homebrew/bin")
    });
    assert_eq!(merged, "/zzz:/usr/bin:/opt/homebrew/bin");
}

#[test]
fn does_not_duplicate_a_dir_already_in_path() {
    let merged = merge_path("/usr/local/bin:/usr/bin", "/Users/me", all_exist);
    assert_eq!(merged.matches("/usr/local/bin").count(), 1);
    assert!(merged.starts_with("/usr/local/bin:/usr/bin:"));
}

#[test]
fn removes_duplicate_current_entries_keeping_first() {
    let merged = merge_path("/usr/bin:/bin:/usr/bin", "/Users/me", none_exist);
    assert_eq!(merged, "/usr/bin:/bin");
}

#[test]
fn drops_empty_entries() {
    let merged = merge_path("::/usr/bin:", "/Users/me", none_exist);
    assert_eq!(merged, "/usr/bin");
}

#[test]
fn skips_home_dirs_when_home_is_empty() {
    let merged = merge_path("/usr/bin", "", all_exist);
    assert_eq!(
        merged,
        "/usr/bin:/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:/nix/var/nix/profiles/default/bin"
    );
}

#[test]
fn checks_home_dirs_under_home() {
    let merged = merge_path("", "/Users/me", |p| p == Path::new("/Users/me/.cargo/bin"));
    assert_eq!(merged, "/Users/me/.cargo/bin");
}
