//! Posts key events to the frontmost app. The logic that picks the key lives in `domain/hotkey.rs`.
//!
//! The hold API lives in `keyboard/os.rs`. CoreGraphics posting lives in `keyboard/os/post.rs`.

mod os;

pub use os::{
    hold_begin, hold_end, hold_touch, install_panic_release, push_to_talk_start, release_all, tap,
};
