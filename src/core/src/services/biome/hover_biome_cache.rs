use std::num::NonZeroUsize;

use lru::LruCache;

use crate::structs::biome::{BiomeKey, RegionBiomes};
use crate::structs::pos::{ChunkPos, RegionPos};

// the biome palettes of regions read from disk for the world map's hover info; bounded, and the
// least recently looked-up region goes first
pub struct HoverBiomeCache {
    regions: LruCache<RegionPos, RegionBiomes>,
}

impl HoverBiomeCache {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self { regions: LruCache::new(capacity) }
    }

    // outer None: the region is not cached; Some(None): it is, and holds no biome for the block
    pub fn lookup(&mut self, block_x: i32, block_z: i32, dim: i32) -> Option<Option<BiomeKey>> {
        let chunk = ChunkPos::from_block(block_x, block_z, dim);
        let biomes = self.regions.get(&chunk.region())?;
        let column = (block_z.rem_euclid(16) * 16 + block_x.rem_euclid(16)) as usize;
        Some(biomes.key(chunk.local_in_region(), column))
    }

    pub fn insert(&mut self, region: RegionPos, biomes: RegionBiomes) {
        self.regions.put(region, biomes);
    }

    pub fn contains(&self, region: RegionPos) -> bool {
        self.regions.contains(&region)
    }

    // a region explored since it was read is read again on its next hover
    pub fn forget(&mut self, region: RegionPos) {
        self.regions.pop(&region);
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.regions.cap().get()
    }

    pub fn clear(&mut self) {
        self.regions.clear();
    }
}
