use super::is_valid_tty;

/// iTerm2 escape that plays a fireworks animation at the pane's cursor. It changes no pane state.
const FIREWORKS: &[u8] = b"\x1b]1337;RequestAttention=fireworks\x07";

/// The device path and the bytes to write to it, so iTerm2 shows the fireworks on `tty`. None
/// unless `tty` is `ttys` followed by digits only, because the path goes to a device open.
pub fn highlight_target(tty: &str) -> Option<(String, &'static [u8])> {
    is_valid_tty(tty).then(|| (format!("/dev/{tty}"), FIREWORKS))
}
