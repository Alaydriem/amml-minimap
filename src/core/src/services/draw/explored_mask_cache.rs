use std::num::NonZeroUsize;

use lru::LruCache;

use crate::structs::pos::RegionPos;
use crate::structs::tile::Tile;

// each region's explored-chunk mask, read from its tile and kept until the tile's version moves;
// at capacity the least recently used region is dropped
pub struct ExploredMaskCache {
    masks: LruCache<RegionPos, (u64, [u64; 4])>,
}

impl ExploredMaskCache {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN);
        Self { masks: LruCache::new(capacity) }
    }

    // the region's mask; `tile` is None when the tile is not resident or its lock is busy, and the
    // last mask read is kept then
    pub fn mask(&mut self, region: RegionPos, tile: Option<&Tile>) -> [u64; 4] {
        match tile {
            Some(tile) => {
                if let Some((version, mask)) = self.masks.get(&region)
                    && *version == tile.version()
                {
                    return *mask;
                }
                let mask = Self::read(tile);
                self.masks.put(region, (tile.version(), mask));
                mask
            }
            None => self.masks.get(&region).map_or([0; 4], |(_, mask)| *mask),
        }
    }

    pub fn len(&self) -> usize {
        self.masks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.masks.is_empty()
    }

    pub fn clear(&mut self) {
        self.masks.clear();
    }

    // a chunk is explored when the pixel at its middle has been baked (is not transparent)
    pub fn read(tile: &Tile) -> [u64; 4] {
        let mut mask = [0u64; 4];
        for lz in 0..16 {
            for lx in 0..16 {
                let alpha = tile.pixel(lx * 16 + 8, lz * 16 + 8) >> 24;
                if alpha != 0 {
                    let bit = lz * 16 + lx;
                    mask[bit / 64] |= 1 << (bit % 64);
                }
            }
        }
        mask
    }
}
