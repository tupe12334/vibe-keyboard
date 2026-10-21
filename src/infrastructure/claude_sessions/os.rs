//! Process spawns for `ps`, `lsof`, `osascript` and `open`. Excluded from the coverage gate; the
//! logic that decides what to do with the output lives in `claude_sessions.rs`.

use std::time::Duration;

mod blink;
mod discovery;
mod front_tty;
mod fullscreen;
mod summary;
pub(super) mod tab_scripts;
mod terminal;
mod tty_age;
pub use blink::blink_session;
pub use discovery::list_sessions;
pub use front_tty::iterm_moved_from;
pub use fullscreen::full_screen_session;
pub use terminal::{focus_session, open_new_session, send_prompt};

/// Budget for the `ps` call that starts one discovery pass.
const PS_TIMEOUT: Duration = Duration::from_millis(150);

/// Hard limit for one `osascript` focus call. Terminal apps can be slow to answer when they activate.
const FOCUS_TIMEOUT: Duration = Duration::from_secs(5);
