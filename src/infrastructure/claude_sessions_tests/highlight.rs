use super::*;

#[test]
fn highlight_writes_the_iterm_fireworks_escape() {
    let (path, bytes) = highlight_target("ttys012").unwrap_or_default();
    assert_eq!(path, "/dev/ttys012");
    assert_eq!(bytes, b"\x1b]1337;RequestAttention=fireworks\x07");
}

#[test]
fn highlight_refuses_anything_but_ttys_digits() {
    for tty in [
        "",
        "ttys",
        "tty12",
        "ttys1/../x",
        "ttys12\n",
        "ttys1 2",
        "/dev/ttys1",
    ] {
        assert_eq!(highlight_target(tty), None, "{tty:?}");
    }
}
