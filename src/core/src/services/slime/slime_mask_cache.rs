use std::num::NonZeroUsize;

use lru::LruCache;

use crate::services::slime::SlimeChunks;

// region slime masks, bounded; the least recently used region goes first. A miss is computed only
// while the frame's budget lasts, so a zoomed-out frame stays cheap
pub struct SlimeMaskCache {
    masks: LruCache<(i32, i32), [u64; 4]>,
}

impl SlimeMaskCache {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self { masks: LruCache::new(capacity) }
    }

    pub fn mask(&mut self, region_x: i32, region_z: i32, budget: &mut usize) -> Option<[u64; 4]> {
        if let Some(mask) = self.masks.get(&(region_x, region_z)) {
            return Some(*mask);
        }
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        let mask = SlimeChunks::region_mask(region_x, region_z);
        self.masks.put((region_x, region_z), mask);
        Some(mask)
    }

    pub fn len(&self) -> usize {
        self.masks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.masks.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.masks.cap().get()
    }
}
