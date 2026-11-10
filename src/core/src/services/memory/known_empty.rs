use std::num::NonZeroUsize;

use lru::LruCache;

use crate::structs::pos::RegionPos;

// regions confirmed to have no data on disk; bounded so world map panning cannot grow it
pub struct KnownEmptySet {
    regions: LruCache<RegionPos, ()>,
}

impl KnownEmptySet {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self { regions: LruCache::new(capacity) }
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub fn insert(&mut self, region: RegionPos) {
        self.regions.put(region, ());
    }

    pub fn contains(&self, region: RegionPos) -> bool {
        self.regions.contains(&region)
    }

    pub fn clear(&mut self) {
        self.regions.clear();
    }

    pub fn remove(&mut self, region: RegionPos) {
        self.regions.pop(&region);
    }
}
