use super::support::{online, routine};
use super::*;

#[test]
fn list_pages_forward_and_back_and_stop_at_edges() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::MoadimList {
        list: online(11),
        page: 0,
    });
    assert!(nav.can_forward());
    nav.forward();
    assert!(matches!(nav.current(), Screen::MoadimList { page, .. } if *page == 1));
    assert!(!nav.can_forward());
    nav.forward();
    assert!(matches!(nav.current(), Screen::MoadimList { page, .. } if *page == 1));
    nav.back();
    assert!(matches!(nav.current(), Screen::MoadimList { page, .. } if *page == 0));
    assert!(nav.is_moadim());
}

#[test]
fn previous_page_moves_a_routine_list_back() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::MoadimList {
        list: online(11),
        page: 1,
    });
    nav.previous_page();
    assert!(matches!(nav.current(), Screen::MoadimList { page, .. } if *page == 0));
    nav.previous_page();
    assert!(matches!(nav.current(), Screen::MoadimList { page, .. } if *page == 0));
}

#[test]
fn pending_and_offline_lists_have_no_pages() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::MoadimList {
        list: MoadimList::Pending,
        page: 0,
    });
    assert!(!nav.can_forward());
    nav.set_moadim(MoadimList::Offline);
    nav.forward();
    assert!(matches!(nav.current(), Screen::MoadimList { page, .. } if *page == 0));
}

#[test]
fn set_moadim_reports_change_and_clamps_page() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::MoadimList {
        list: online(11),
        page: 1,
    });
    assert!(!nav.set_moadim(online(11)));
    assert!(nav.set_moadim(online(3)));
    assert!(matches!(nav.current(), Screen::MoadimList { page, .. } if *page == 0));
    assert!(nav.set_moadim(MoadimList::Offline));
    assert!(!nav.set_moadim(MoadimList::Offline));
}

#[test]
fn set_moadim_refreshes_the_open_routine_by_id() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::MoadimRoutine {
        routine: routine(2),
        flash: None,
    });
    assert!(!nav.set_moadim(MoadimList::Offline));
    assert!(!nav.set_moadim(online(1)), "routine 2 is not in the list");
    assert!(!nav.set_moadim(online(3)), "same routine, nothing new");
    let mut changed = routine(2);
    changed.enabled = false;
    assert!(nav.set_moadim(MoadimList::Online(vec![changed.clone()])));
    assert!(matches!(nav.current(), Screen::MoadimRoutine { routine, .. } if *routine == changed));
}

#[test]
fn set_moadim_ignores_other_screens() {
    let mut nav = NavigationStack::new(0, 1);
    assert!(!nav.set_moadim(online(2)));
    assert!(!nav.is_moadim());
}
