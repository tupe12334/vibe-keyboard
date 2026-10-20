//! Push-to-talk hold API for the device loop. Excluded from the coverage gate.
//!
//! Claude Code starts dictation on a held key by counting rapid key-repeat events. So a hold
//! posts key down again, with the auto-repeat flag set, until it ends. The rules live in
//! `domain/ptt`; `hold.rs` owns the guard and `beat.rs` owns the two threads of a hold.

mod accessibility;
mod beat;
mod hold;
mod panic;
mod post;

pub use accessibility::accessibility_trusted;
pub use hold::{hold_begin, hold_end, hold_touch, push_to_talk_start, release_all};
pub use panic::install_panic_release;
pub use post::tap;
