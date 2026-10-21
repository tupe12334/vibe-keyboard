use std::ffi::OsStr;
use std::path::Path;

use super::{console_args, is_show_log, plain_args, MENU_ID};

#[test]
fn only_the_show_log_id_maps_to_the_show_log_action() {
    assert!(is_show_log(MENU_ID));
    assert!(!is_show_log("open"));
    assert!(!is_show_log("accessibility"));
    assert!(!is_show_log(""));
}

#[test]
fn console_args_open_the_path_in_console() {
    let args = console_args(Path::new("/home/u/.config/vibe-keyboard/app.log"));
    assert_eq!(
        args,
        [
            OsStr::new("-a"),
            OsStr::new("Console"),
            OsStr::new("/home/u/.config/vibe-keyboard/app.log"),
        ]
    );
}

#[test]
fn plain_args_open_the_path_alone() {
    let args = plain_args(Path::new("/home/u/.config/vibe-keyboard/app.log"));
    assert_eq!(args, [OsStr::new("/home/u/.config/vibe-keyboard/app.log")]);
}
