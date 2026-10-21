use super::*;

#[test]
#[allow(clippy::expect_used, reason = "test: setup failures should panic")]
fn second_acquire_reports_held_until_first_is_dropped() {
    let dir = std::env::temp_dir().join(format!(
        "vibe-keyboard-lock-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    std::fs::create_dir_all(&dir).expect("create test dir");
    let path = dir.join(LOCK_FILE);

    let first = acquire(&path).expect("first acquire succeeds");
    assert!(matches!(acquire(&path), Err(AcquireError::Held)));

    drop(first); // Fork/exec race: a concurrent fork holds our fd until exec, so retry.
    let second = (0..100).find_map(|_| {
        std::thread::sleep(std::time::Duration::from_millis(10));
        acquire(&path).ok()
    });
    assert!(second.is_some(), "lock is free after the holder drops it");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn lock_io_error_is_not_reported_as_held() {
    let err = AcquireError::from(TryLockError::Error(io::Error::other("disk")));
    assert!(matches!(err, AcquireError::Io(_)));
}

#[test]
fn default_path_is_the_lock_file() {
    assert!(default_lock_path().ends_with(LOCK_FILE));
}
