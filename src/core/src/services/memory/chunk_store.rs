use crate::structs::biome::BiomeKey;
use std::collections::HashMap;

use crate::services::memory::chunk_slot::ChunkSlot;
use crate::structs::chunk::{
    ChunkSurface, ColumnSample, DirtyFlags, EvictedChunk, ShadowMask, SubmitResult, SurfaceMerge,
};
use crate::structs::pos::{ChunkPos, RegionPos};
use crate::traits::{BakeEnvironment, HeightRanges, ScanHistory};

// fixed-capacity slab of chunk surfaces; when full, the chunks farthest from the player go first
pub struct ChunkStore {
    slots: Vec<ChunkSlot>,
    index: HashMap<ChunkPos, usize>,
    free: Vec<usize>,
    scratch: Vec<(i64, usize)>,
    capacity: usize,
}

impl ChunkStore {
    pub const MIN_CAPACITY: usize = 16;
    const INDEX_OVERHEAD_BYTES: usize = 64;

    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(Self::MIN_CAPACITY);
        Self {
            slots: Vec::with_capacity(capacity),
            index: HashMap::with_capacity(capacity),
            free: Vec::with_capacity(capacity),
            scratch: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn capacity_for_budget(budget_bytes: usize, scale: u8) -> usize {
        let slot = std::mem::size_of::<ChunkSlot>() + ShadowMask::byte_len(scale) + Self::INDEX_OVERHEAD_BYTES;
        (budget_bytes / slot).max(Self::MIN_CAPACITY)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn contains(&self, pos: ChunkPos) -> bool {
        self.index.contains_key(&pos)
    }

    // the stored map height of one block column, if its chunk is held and the column was probed
    pub fn column_height(&self, block_x: i32, block_z: i32, dim: i32) -> Option<i16> {
        let slot = self.slot(ChunkPos::from_block(block_x, block_z, dim))?;
        let index = ChunkSurface::index(block_x.rem_euclid(16) as usize, block_z.rem_euclid(16) as usize);
        (!slot.surface.is_skipped(index)).then_some(slot.surface.height[index])
    }

    // None when the chunk is not held, the column was never probed, or its biome is unknown
    pub fn column_biome(&self, block_x: i32, block_z: i32, dim: i32) -> Option<BiomeKey> {
        let slot = self.slot(ChunkPos::from_block(block_x, block_z, dim))?;
        let index = ChunkSurface::index(block_x.rem_euclid(16) as usize, block_z.rem_euclid(16) as usize);
        if slot.surface.is_skipped(index) {
            return None;
        }
        slot.surface.biome.key(index)
    }

    pub fn dirty(&self, pos: ChunkPos) -> Option<DirtyFlags> {
        self.slot(pos).map(|s| s.dirty)
    }

    pub fn is_journal_dirty(&self, pos: ChunkPos) -> bool {
        self.slot(pos).is_some_and(|s| s.journal_dirty)
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.index.clear();
        self.free.clear();
    }

    // merges a fresh probe; a chunk with no written column is not stored
    pub fn submit(
        &mut self,
        pos: ChunkPos,
        incoming: &ChunkSurface,
        tick: u64,
        player: ChunkPos,
        evicted: &mut Vec<EvictedChunk>,
    ) -> SubmitResult {
        let merge = match self.slot(pos) {
            Some(slot) => slot.surface.merge(incoming),
            None => ChunkSurface::empty().merge(incoming),
        };
        if !merge.surface.has_base_data() {
            return Self::result(&merge);
        }
        let slot = match self.index.get(&pos) {
            Some(&i) => i,
            None => self.allocate(pos, player, evicted),
        };
        Self::apply(&mut self.slots[slot], &merge, Some(tick));
        Self::result(&merge)
    }

    // inserts a surface read back from disk; a chunk already in memory is newer and is kept
    pub fn restore(
        &mut self,
        pos: ChunkPos,
        surface: &ChunkSurface,
        player: ChunkPos,
        evicted: &mut Vec<EvictedChunk>,
    ) -> bool {
        if self.index.contains_key(&pos) || !surface.has_base_data() {
            return false;
        }
        let slot = self.allocate(pos, player, evicted);
        self.slots[slot].surface = *surface;
        true
    }

    // None when the chunk is not stored or has no base data; the caller then probes the whole chunk
    pub fn submit_column(
        &mut self,
        pos: ChunkPos,
        index: usize,
        column: ColumnSample,
        biome: Option<BiomeKey>,
    ) -> Option<SubmitResult> {
        let &i = self.index.get(&pos)?;
        let merge = self.slots[i].surface.merge_column(index, column, biome)?;
        Self::apply(&mut self.slots[i], &merge, None);
        Some(Self::result(&merge))
    }

    pub fn mark_full(&mut self, pos: ChunkPos) -> bool {
        match self.slot_mut(pos) {
            Some(slot) if slot.surface.has_base_data() => {
                slot.dirty.mark_full();
                true
            }
            _ => false,
        }
    }

    // a chunk that is already full-dirty needs no soft mark
    pub fn mark_soft(&mut self, pos: ChunkPos) -> bool {
        match self.slot_mut(pos) {
            Some(slot) if slot.surface.has_base_data() && !slot.dirty.is_full() => {
                slot.dirty.mark_soft();
                true
            }
            _ => false,
        }
    }

    pub fn clear_full_if(&mut self, pos: ChunkPos, epoch: u32) -> bool {
        self.slot_mut(pos).is_some_and(|s| s.dirty.clear_full_if(epoch))
    }

    pub fn clear_soft_if(&mut self, pos: ChunkPos, epoch: u32) -> bool {
        self.slot_mut(pos).is_some_and(|s| s.dirty.clear_soft_if(epoch))
    }

    // returns true when the stored mask changed
    pub fn store_mask(&mut self, pos: ChunkPos, mask: &ShadowMask) -> bool {
        let Some(slot) = self.slot_mut(pos) else { return false };
        if slot.mask == *mask {
            return false;
        }
        slot.mask.copy_from(mask);
        true
    }

    // bumped by every submit that changed the chunk; a journal ack carries the version it wrote
    pub fn journal_version(&self, pos: ChunkPos) -> Option<u64> {
        self.slot(pos).map(|s| s.journal_version)
    }

    // clears the journal mark only when no change arrived after `version` was journaled
    pub fn mark_journaled_if(&mut self, pos: ChunkPos, version: u64) -> bool {
        match self.slot_mut(pos) {
            Some(slot) if slot.journal_version == version => {
                slot.journal_dirty = false;
                true
            }
            _ => false,
        }
    }

    // every chunk not yet journaled, marked journaled as it is handed out; used when a world is left
    pub fn drain_journal_dirty(&mut self, out: &mut Vec<EvictedChunk>) {
        for &i in self.index.values() {
            let slot = &mut self.slots[i];
            if slot.journal_dirty {
                slot.journal_dirty = false;
                out.push(EvictedChunk { pos: slot.pos, surface: slot.surface });
            }
        }
    }

    pub fn mark_journaled(&mut self, pos: ChunkPos) {
        if let Some(slot) = self.slot_mut(pos) {
            slot.journal_dirty = false;
        }
    }

    // full holds full-dirty chunks; soft holds chunks that are only soft-dirty
    pub fn dirty_in_region(&self, region: RegionPos, full: &mut Vec<ChunkPos>, soft: &mut Vec<ChunkPos>) {
        full.clear();
        soft.clear();
        for lz in 0..16 {
            for lx in 0..16 {
                let pos = region.chunk(lx, lz);
                let Some(slot) = self.slot(pos) else { continue };
                if slot.dirty.is_full() {
                    full.push(pos);
                } else if slot.dirty.is_soft() {
                    soft.push(pos);
                }
            }
        }
    }

    fn slot(&self, pos: ChunkPos) -> Option<&ChunkSlot> {
        self.index.get(&pos).map(|&i| &self.slots[i])
    }

    fn slot_mut(&mut self, pos: ChunkPos) -> Option<&mut ChunkSlot> {
        self.index.get(&pos).map(|&i| &mut self.slots[i])
    }

    fn apply(slot: &mut ChunkSlot, merge: &SurfaceMerge, tick: Option<u64>) {
        slot.surface = merge.surface;
        if !merge.changes.is_empty() {
            slot.journal_dirty = true;
            slot.journal_version += 1;
        }
        if let Some(tick) = tick
            && !merge.needs_retry
        {
            slot.last_scan = Some(tick);
        }
    }

    fn result(merge: &SurfaceMerge) -> SubmitResult {
        SubmitResult {
            changes: merge.changes,
            first_load: merge.first_load,
            previous_range: merge.previous_range,
            new_range: merge.surface.height_range(),
            needs_retry: merge.needs_retry,
        }
    }

    fn allocate(&mut self, pos: ChunkPos, player: ChunkPos, evicted: &mut Vec<EvictedChunk>) -> usize {
        if self.index.len() >= self.capacity {
            self.evict_farthest(player, evicted);
        }
        let slot = match self.free.pop() {
            Some(i) => {
                self.slots[i] = ChunkSlot::new(pos);
                i
            }
            None => {
                self.slots.push(ChunkSlot::new(pos));
                self.slots.len() - 1
            }
        };
        self.index.insert(pos, slot);
        slot
    }

    // drops the farthest sixteenth of the store in one pass, so eviction cost is amortised
    fn evict_farthest(&mut self, player: ChunkPos, evicted: &mut Vec<EvictedChunk>) {
        self.scratch.clear();
        for &i in self.index.values() {
            let slot = &self.slots[i];
            let distance = if slot.pos.dim == player.dim { slot.pos.distance_squared(&player) } else { i64::MAX };
            self.scratch.push((distance, i));
        }
        let batch = (self.capacity / 16).max(1).min(self.scratch.len());
        let cut = self.scratch.len() - batch;
        self.scratch.select_nth_unstable_by_key(cut, |&(distance, _)| distance);
        for k in cut..self.scratch.len() {
            let i = self.scratch[k].1;
            let slot = &self.slots[i];
            if slot.journal_dirty {
                evicted.push(EvictedChunk { pos: slot.pos, surface: slot.surface });
            }
            self.index.remove(&slot.pos);
            self.free.push(i);
        }
    }
}

impl ScanHistory for ChunkStore {
    fn last_scan(&self, pos: ChunkPos) -> Option<u64> {
        self.slot(pos).and_then(|s| s.last_scan)
    }
}

impl HeightRanges for ChunkStore {
    fn height_range(&self, pos: ChunkPos) -> Option<(i16, i16)> {
        self.slot(pos).and_then(|s| s.surface.height_range())
    }
}

impl BakeEnvironment for ChunkStore {
    fn surface(&self, pos: ChunkPos) -> Option<&ChunkSurface> {
        self.slot(pos).map(|s| &s.surface).filter(|s| s.has_base_data())
    }

    fn mask(&self, pos: ChunkPos) -> Option<&ShadowMask> {
        self.slot(pos).map(|s| &s.mask).filter(|m| m.scale() != 0)
    }
}
