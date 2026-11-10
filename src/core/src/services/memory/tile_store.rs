use lru::LruCache;

use crate::errors::TileError;
use crate::structs::pos::{ChunkPos, RegionPos};
use crate::structs::tile::Tile;

// LRU of region tiles; clean tiles are evicted first and dirty tiles are never dropped silently
pub struct TileStore {
    tiles: LruCache<RegionPos, Tile>,
    capacity: usize,
    hard_cap: usize,
    flush_requested: bool,
}

impl TileStore {
    pub const MIN_CAPACITY: usize = 4;

    pub fn new(capacity: usize, overrun_percent: usize) -> Self {
        let capacity = capacity.max(Self::MIN_CAPACITY);
        Self {
            tiles: LruCache::unbounded(),
            capacity,
            hard_cap: capacity + capacity * overrun_percent / 100,
            flush_requested: false,
        }
    }

    pub fn capacity_for_budget(budget_bytes: usize) -> usize {
        (budget_bytes / Tile::BYTES).max(Self::MIN_CAPACITY)
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
        self.flush_requested = false;
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    pub fn overrun(&self) -> usize {
        self.tiles.len().saturating_sub(self.capacity)
    }

    // returns and clears the request raised when only dirty tiles were left to evict
    pub fn take_flush_request(&mut self) -> bool {
        std::mem::take(&mut self.flush_requested)
    }

    // marks the tile as recently used
    pub fn get(&mut self, pos: RegionPos) -> Option<&Tile> {
        self.tiles.get(&pos)
    }

    pub fn peek(&self, pos: RegionPos) -> Option<&Tile> {
        self.tiles.peek(&pos)
    }

    // `evicted` receives dirty tiles that had to leave memory; the caller must persist them.
    // Returns true when the tile was created, so the caller can ask for its stored pixels.
    pub fn write_chunk(
        &mut self,
        chunk: ChunkPos,
        pixels: &[u32; 256],
        settings_hash: u64,
        evicted: &mut Vec<(RegionPos, Tile)>,
    ) -> bool {
        let region = chunk.region();
        let created = !self.tiles.contains(&region);
        if created {
            self.make_room(evicted);
            self.tiles.put(region, Tile::new(settings_hash));
        }
        let (lx, lz) = chunk.local_in_region();
        if let Some(tile) = self.tiles.get_mut(&region) {
            tile.write_chunk(lx, lz, pixels, settings_hash);
        }
        created
    }

    pub fn insert_loaded(
        &mut self,
        region: RegionPos,
        pixels: &[u8],
        settings_hash: u64,
        evicted: &mut Vec<(RegionPos, Tile)>,
    ) -> Result<(), TileError> {
        if let Some(tile) = self.tiles.get_mut(&region) {
            tile.merge_under(pixels)?;
            return Ok(());
        }
        let mut tile = Tile::new(settings_hash);
        tile.replace_pixels(pixels, settings_hash)?;
        self.make_room(evicted);
        self.tiles.put(region, tile);
        Ok(())
    }

    // the regions of one dimension that have a tile in memory
    pub fn regions(&self, dim: i32, out: &mut Vec<RegionPos>) {
        out.clear();
        out.extend(self.tiles.iter().map(|(p, _)| *p).filter(|p| p.dim == dim));
    }

    pub fn dirty_regions(&self, out: &mut Vec<RegionPos>) {
        out.clear();
        out.extend(self.tiles.iter().filter(|(_, t)| t.is_dirty()).map(|(p, _)| *p));
    }

    pub fn mark_clean_if(&mut self, region: RegionPos, version: u64) -> bool {
        self.tiles.peek_mut(&region).is_some_and(|t| t.mark_clean_if(version))
    }

    fn make_room(&mut self, evicted: &mut Vec<(RegionPos, Tile)>) {
        if self.tiles.len() < self.capacity {
            return;
        }
        let clean = self.tiles.iter().rev().find(|(_, t)| !t.is_dirty()).map(|(p, _)| *p);
        if let Some(pos) = clean {
            self.tiles.pop(&pos);
            return;
        }
        self.flush_requested = true;
        if self.tiles.len() < self.hard_cap {
            return;
        }
        if let Some(entry) = self.tiles.pop_lru() {
            evicted.push(entry);
        }
    }
}
