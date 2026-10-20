//! Physical memory use: the value the sampler reads, and its percent.

/// Used and total physical memory, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Memory {
    pub used: u64,
    pub total: u64,
}

/// Memory use as a whole percent, 0 to 100. Missing memory, or a zero total, gives 0.
pub fn memory_percent(memory: Option<Memory>) -> u8 {
    let Some(Memory { used, total }) = memory else {
        return 0;
    };
    if total == 0 {
        return 0;
    }
    let (used, total) = (u128::from(used.min(total)), u128::from(total));
    // Round to the nearest percent, so 1 byte short of full still reads 100 only when full.
    ((used * 100 + total / 2) / total) as u8
}
