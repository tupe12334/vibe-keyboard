use super::*;

#[test]
fn new_initializes_defaults() {
    let s = AppState::new(3);
    assert_eq!(s.total_pages, 3);
    assert_eq!(s.pressed_key, None);
    assert!(!s.loading);
    assert!(s.log.is_empty());
    assert!(s.actions.is_empty());
    assert!(s.nav_can_back);
    assert!(!s.nav_can_out);
    assert!(s.nav_can_forward);
}

#[test]
fn push_log_adds_entries() {
    let mut s = AppState::new(1);
    s.push_log("hello".into());
    assert_eq!(s.log.len(), 1);
    assert_eq!(s.log[0], "hello");
}

#[test]
fn push_log_evicts_oldest_at_capacity() {
    let mut s = AppState::new(1);
    for i in 0..10 {
        s.push_log(format!("msg{i}"));
    }
    assert_eq!(s.log.len(), 10);
    // adding one more should evict the first
    s.push_log("overflow".into());
    assert_eq!(s.log.len(), 10);
    assert_eq!(s.log[0], "msg1");
    assert_eq!(s.log[9], "overflow");
}
