#![allow(clippy::expect_used)]

use super::*;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc;
use std::time::Duration;

/// A stand-in controller: runs until its shutdown flag is set. Touches no device.
fn fake_running() -> Running {
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&shutdown);
    let thread = std::thread::spawn(move || {
        while !flag.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(2));
        }
    });
    Running { shutdown, thread }
}

#[test]
fn start_with_never_relocks_and_relaunches_only_after_exit() {
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let running: Mutex<Option<Running>> = Mutex::new(None);
        let launches = AtomicUsize::new(0);
        let launch = || {
            launches.fetch_add(1, Ordering::SeqCst);
            Ok(fake_running())
        };
        // First start launches. Second start, while the thread lives, does not.
        assert_eq!(start_with(&running, launch), Ok(()));
        assert_eq!(start_with(&running, launch), Ok(()));
        assert_eq!(launches.load(Ordering::SeqCst), 1);

        // Stop the thread and wait for it to end. Then a start launches again.
        let shutdown = running
            .lock()
            .expect("lock")
            .as_ref()
            .map(|r| Arc::clone(&r.shutdown))
            .expect("running");
        shutdown.store(true, Ordering::Relaxed);
        while running
            .lock()
            .expect("lock")
            .as_ref()
            .is_some_and(Running::is_alive)
        {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(start_with(&running, launch), Ok(()));
        assert_eq!(launches.load(Ordering::SeqCst), 2);

        // A refused launch leaves the slot as it was: no thread, error returned.
        let refused = start_with(&Mutex::new(None), || Err("held".to_string()));
        assert_eq!(refused, Err("held".to_string()));

        let _ = done_tx.send(());
    });
    // A re-entrant lock would hang here instead of failing. The timeout makes it fail.
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("start_with hung: re-entrant lock?");
}
