//! Manual check: prints each frontmost-app change for 10 seconds.
//!
//! Run with `cargo run --example frontmost`, then switch apps.

#![allow(clippy::print_stdout, reason = "manual check: prints to the terminal")] // ponytail: manual check, prints to the terminal

// ponytail: the example uses only `watch`; the rest is dead here.
#[allow(dead_code, reason = "the example uses only `watch`")]
#[path = "../src/infrastructure/frontmost.rs"]
mod frontmost;

use std::time::{Duration, Instant};

fn main() {
    let rx = frontmost::watch(Duration::from_millis(500));
    let end = Instant::now() + Duration::from_secs(10);
    while let Ok(id) = rx.recv_timeout(end.saturating_duration_since(Instant::now())) {
        println!("{id}");
    }
}
