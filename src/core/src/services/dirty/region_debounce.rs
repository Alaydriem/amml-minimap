// waits DELAY ticks after the last mark, but never more than MAX_DELAY ticks after the first
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionDebounce {
    delay: u8,
    max_delay: u8,
}

impl RegionDebounce {
    pub const DELAY: u8 = 10;
    pub const MAX_DELAY: u8 = 60;

    pub fn new() -> Self {
        Self { delay: Self::DELAY, max_delay: Self::MAX_DELAY }
    }

    pub fn mark(&mut self) {
        if self.delay > 0 {
            self.delay = Self::DELAY;
        }
    }

    // true on the tick the region should be dispatched
    pub fn tick(&mut self) -> bool {
        if self.delay == 0 || self.max_delay == 0 {
            return false;
        }
        self.delay -= 1;
        self.max_delay -= 1;
        self.delay == 0 || self.max_delay == 0
    }
}

impl Default for RegionDebounce {
    fn default() -> Self {
        Self::new()
    }
}
