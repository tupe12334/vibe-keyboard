pub struct Navigator {
    current: usize,
    count: usize,
}

impl Navigator {
    pub fn new(count: usize) -> Self {
        assert!(count > 0, "must have at least one page");
        Self { current: 0, count }
    }

    pub fn current(&self) -> usize {
        self.current
    }

    /// Go back one page (wraps around). Returns the new page index.
    pub fn back(&mut self) -> usize {
        self.current = if self.current == 0 { self.count - 1 } else { self.current - 1 };
        self.current
    }

    /// Go forward one page (wraps around). Returns the new page index.
    pub fn forward(&mut self) -> usize {
        self.current = (self.current + 1) % self.count;
        self.current
    }
}
