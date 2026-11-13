use super::*;
use crate::domain::navigation::NavigationStack;
use std::sync::mpsc::{channel, Sender};

fn project() -> CentyProject {
    CentyProject {
        name: "p".into(),
        org: "o".into(),
        path: None,
        url: "https://app.centy.io/o/p".into(),
    }
}

/// A pending fetch started on the given screen, with the sender that completes it.
fn pending_on(nav: &NavigationStack) -> (Option<Pending>, Sender<Fetched>) {
    let (tx, rx) = channel();
    (
        Some(Pending {
            rx,
            from: nav.screen_id(),
        }),
        tx,
    )
}

fn main_page() -> NavigationStack {
    NavigationStack::new(0, 2)
}

#[test]
fn no_projects_keeps_the_current_page() {
    assert!(next_screen(Fetched::Projects(vec![])).is_none());
}

#[test]
fn projects_open_the_project_list_on_its_first_page() {
    assert!(matches!(
        next_screen(Fetched::Projects(vec![project()])),
        Some(Screen::CentyProjectList { page: 0, .. })
    ));
}

#[test]
fn sessions_always_open_the_sessions_page() {
    assert!(matches!(
        next_screen(Fetched::Sessions(vec![])),
        Some(Screen::ClaudeSessions { page: 0, .. })
    ));
}

#[test]
fn empty_channel_keeps_the_fetch_waiting() {
    let nav = main_page();
    let (mut slot, _tx) = pending_on(&nav);
    assert!(matches!(
        take_ready(&mut slot, nav.screen_id()),
        Ready::Waiting
    ));
    assert!(slot.is_some());
}

#[test]
fn lost_fetch_clears_the_slot_and_shows_nothing() {
    let nav = main_page();
    let (mut slot, tx) = pending_on(&nav);
    drop(tx);
    assert!(matches!(
        take_ready(&mut slot, nav.screen_id()),
        Ready::Done(None)
    ));
    assert!(slot.is_none());
}

#[test]
fn landed_fetch_on_the_same_screen_opens_its_page() {
    let nav = main_page();
    let (mut slot, tx) = pending_on(&nav);
    let _ = tx.send(Fetched::Sessions(vec![]));
    assert!(matches!(
        take_ready(&mut slot, nav.screen_id()),
        Ready::Done(Some(Screen::ClaudeSessions { .. }))
    ));
    assert!(slot.is_none());
}

#[test]
fn result_for_a_screen_the_user_left_is_dropped() {
    let mut nav = main_page();
    let (mut slot, tx) = pending_on(&nav);
    let _ = tx.send(Fetched::Projects(vec![project()]));
    nav.push(Screen::InputNumber {
        value: String::new(),
    });
    assert!(matches!(
        take_ready(&mut slot, nav.screen_id()),
        Ready::Done(None)
    ));
    assert!(slot.is_none());
}
