use super::Disk;

/// Mount points that hold the user's data. On APFS, `/` is the sealed system snapshot and
/// `/System/Volumes/Data` is the writable volume. Both report the same container.
pub fn is_candidate_mount(mount: &str) -> bool {
    matches!(mount, "/" | "/System/Volumes/Data")
}

/// Picks the candidate with the most free space: the volume a user can still fill.
pub fn choose_disk(candidates: &[Disk]) -> Option<Disk> {
    candidates.iter().copied().max_by_key(|disk| disk.free)
}
