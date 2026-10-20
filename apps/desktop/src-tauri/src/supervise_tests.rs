#![allow(clippy::expect_used)]

use crate::lifecycle::{stop_running, Running};
use crate::supervise::spawn_controller;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

/// Work for the main thread. It is queued, and runs only when the main thread gets to it.
type MainWork = Box<dyn FnOnce() + Send>;

#[test]
fn stop_finishes_while_the_exit_report_is_queued_for_the_main_thread() {
    let (main_tx, main_rx) = mpsc::channel::<MainWork>();
    let (reported_tx, reported_rx) = mpsc::channel::<Option<String>>();
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&shutdown);
    let thread = spawn_controller(
        move || {
            while !flag.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(2));
            }
            None
        },
        move |reason| {
            // Like report_exit: queue the main-thread work and return at once.
            let _ = main_tx.send(Box::new(move || {
                let _ = reported_tx.send(reason);
            }));
        },
    );
    let running = Mutex::new(Some(Running { shutdown, thread }));

    // Stop runs off this thread, so a hang shows as a timeout instead of a stuck test.
    let (done_tx, done_rx) = mpsc::channel();
    let stopper = std::thread::spawn(move || {
        stop_running(&running);
        let _ = done_tx.send(());
    });
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("stop hung: the exit report waited for the main thread");
    stopper.join().expect("stopper thread");

    // The main thread runs the queued work now, and sees a clean stop.
    let work = main_rx.try_recv().expect("exit report queued");
    work();
    assert_eq!(reported_rx.try_recv(), Ok(None));
}
