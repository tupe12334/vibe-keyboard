//! Tells a tap from a hold on each deck key. Pure: the caller passes the time in.

mod gesture;

pub use gesture::{Gesture, Held};

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

struct Press {
    since: Instant,
    held: Held,
    holding: bool,
}

/// Keys that are down, with the time each went down.
#[derive(Default)]
pub struct HoldTracker {
    presses: BTreeMap<u8, Press>,
}

impl HoldTracker {
    /// Start tracking `key`. A press of a key that is already down changes nothing.
    pub fn press(&mut self, key: u8, now: Instant, held: Held) {
        self.presses.entry(key).or_insert(Press {
            since: now,
            held,
            holding: false,
        });
    }

    /// Key up. Returns a tap when the key came up before `threshold`. Otherwise it ends the hold,
    /// after a hold start if `poll` has not reported it yet.
    pub fn release(&mut self, key: u8, now: Instant, threshold: Duration) -> Vec<Gesture> {
        let Some(Press {
            since,
            held,
            holding,
        }) = self.presses.remove(&key)
        else {
            return Vec::new();
        };
        if holding {
            vec![Gesture::HoldEnd { key, held }]
        } else if now.duration_since(since) >= threshold {
            vec![
                Gesture::HoldStart {
                    key,
                    held: held.clone(),
                },
                Gesture::HoldEnd { key, held },
            ]
        } else {
            vec![Gesture::Tap { key, held }]
        }
    }

    /// Start the hold of every key that has been down for at least `threshold`.
    pub fn poll(&mut self, now: Instant, threshold: Duration) -> Vec<Gesture> {
        let mut out = Vec::new();
        for (key, press) in &mut self.presses {
            if !press.holding && now.duration_since(press.since) >= threshold {
                press.holding = true;
                out.push(Gesture::HoldStart {
                    key: *key,
                    held: press.held.clone(),
                });
            }
        }
        out
    }

    /// Time until the next key that is down reaches `threshold`. `None` when no key can still become a hold.
    pub fn next_deadline(&self, now: Instant, threshold: Duration) -> Option<Duration> {
        self.presses
            .values()
            .filter(|press| !press.holding)
            .map(|press| (press.since + threshold).saturating_duration_since(now))
            .min()
    }
}

#[cfg(test)]
#[path = "hold_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "hold_keys_tests.rs"]
mod keys_tests;
