use super::Watcher;
use std::cell::Cell;

#[test]
fn tick_sends_first_bundle_then_nothing_for_same_asn() {
    let lookups = Cell::new(0);
    let lookup = |_: &str| {
        lookups.set(lookups.get() + 1);
        Some("com.a".into())
    };
    let mut w = Watcher::default();
    assert_eq!(w.tick(Some("asn1".into()), lookup), Some("com.a".into()));
    assert_eq!(w.tick(Some("asn1".into()), lookup), None);
    assert_eq!(lookups.get(), 1, "lookup must run once per ASN");
}

#[test]
fn tick_looks_up_again_only_when_asn_changes() {
    let mut w = Watcher::default();
    assert_eq!(
        w.tick(Some("asn1".into()), |_| Some("com.a".into())),
        Some("com.a".into())
    );
    assert_eq!(
        w.tick(Some("asn2".into()), |_| Some("com.b".into())),
        Some("com.b".into())
    );
}

#[test]
fn tick_new_asn_with_same_bundle_is_not_sent_again() {
    let mut w = Watcher::default();
    assert_eq!(
        w.tick(Some("asn1".into()), |_| Some("com.a".into())),
        Some("com.a".into())
    );
    assert_eq!(w.tick(Some("asn2".into()), |_| Some("com.a".into())), None);
}

#[test]
fn tick_none_front_keeps_state_and_skips_lookup() {
    let mut w = Watcher::default();
    let lookup = |_: &str| Some("com.a".into());
    assert_eq!(w.tick(Some("asn1".into()), lookup), Some("com.a".into()));
    assert_eq!(w.tick(None, lookup), None);
    assert_eq!(w.asn.as_deref(), Some("asn1"));
}

#[test]
fn tick_retries_lookup_after_a_failed_one() {
    let mut w = Watcher::default();
    assert_eq!(w.tick(Some("asn1".into()), |_| None), None);
    assert_eq!(
        w.tick(Some("asn1".into()), |_| Some("com.a".into())),
        Some("com.a".into())
    );
}
