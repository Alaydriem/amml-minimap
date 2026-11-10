use crate::structs::pos::RegionPos;

// asks for a region only after the cursor has rested on it, so a fast sweep of the mouse over the
// world map does not queue a disk read for every region it crosses
pub struct HoverDebounce {
    current: Option<RegionPos>,
    frames: u32,
    sent: Option<RegionPos>,
}

impl HoverDebounce {
    pub const FRAMES: u32 = 8;

    pub fn new() -> Self {
        Self { current: None, frames: 0, sent: None }
    }

    // true once, on the FRAMES-th frame in a row on the same region, and never for the region last sent
    pub fn observe(&mut self, region: RegionPos) -> bool {
        if self.current == Some(region) {
            self.frames = self.frames.saturating_add(1);
        } else {
            self.current = Some(region);
            self.frames = 1;
        }
        if self.frames == Self::FRAMES && self.sent != Some(region) {
            self.sent = Some(region);
            return true;
        }
        false
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for HoverDebounce {
    fn default() -> Self {
        Self::new()
    }
}
