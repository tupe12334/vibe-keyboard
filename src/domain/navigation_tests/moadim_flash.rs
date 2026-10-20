use super::support::{online, routine};
use super::*;
use std::time::Duration;

#[test]
fn flash_shows_then_clears_after_its_time() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::MoadimRoutine {
        routine: routine(0),
        flash: None,
    });
    let now = Instant::now();
    nav.set_flash(Flash {
        key: 1,
        label: "STARTED",
        until: now + Duration::from_secs(1),
    });
    assert!(!nav.clear_expired_flash(now));
    assert!(nav.clear_expired_flash(now + Duration::from_secs(2)));
    assert!(!nav.clear_expired_flash(now + Duration::from_secs(3)));
}

#[test]
fn flash_is_ignored_off_the_routine_screen() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::MoadimList {
        list: online(1),
        page: 0,
    });
    nav.set_flash(Flash {
        key: 1,
        label: "X",
        until: Instant::now(),
    });
    assert!(!nav.clear_expired_flash(Instant::now()));
}
