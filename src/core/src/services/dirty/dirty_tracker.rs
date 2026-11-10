use std::collections::HashMap;

use crate::services::dirty::{AffectedChunkTracer, RegionDebounce};
use crate::services::memory::ChunkStore;
use crate::structs::LightDirection;
use crate::structs::bake::BakeJob;
use crate::structs::chunk::{ChangeKinds, DirtyEpochs, HeightChange, SubmitResult};
use crate::structs::pos::{BlockRect, ChunkPos, RegionPos};

// turns surface changes into dirty marks, and dirty regions into bake jobs
pub struct DirtyTracker {
    regions: HashMap<RegionPos, RegionDebounce>,
    tracer: AffectedChunkTracer,
    affected: Vec<ChunkPos>,
    fired: Vec<RegionPos>,
    full: Vec<ChunkPos>,
    soft: Vec<ChunkPos>,
    light: LightDirection,
    region_threshold: usize,
}

impl DirtyTracker {
    pub fn new(light: LightDirection, region_threshold: usize) -> Self {
        Self {
            regions: HashMap::new(),
            tracer: AffectedChunkTracer::new(),
            affected: Vec::with_capacity(64),
            fired: Vec::new(),
            full: Vec::with_capacity(256),
            soft: Vec::with_capacity(256),
            light,
            region_threshold,
        }
    }

    pub fn pending_regions(&self) -> usize {
        self.regions.len()
    }

    pub fn clear(&mut self) {
        self.regions.clear();
    }

    // a job that could not be queued: the region is dispatched again after its debounce
    pub fn requeue(&mut self, region: RegionPos) {
        self.touch(region);
    }

    pub fn on_chunk_submit(&mut self, store: &mut ChunkStore, pos: ChunkPos, result: &SubmitResult) {
        // a biome-only change is journaled by the store but changes no baked pixel
        if !result.changes.intersects(ChangeKinds::PIXELS) {
            return;
        }
        self.mark_full(store, pos);
        let Some(new) = result.new_range else { return };
        let height_changed = result.first_load || result.changes.contains(ChangeKinds::HEIGHT);
        if height_changed {
            let (higher, lower) = match (result.first_load, result.previous_range) {
                (false, Some(old)) => (i32::from(old.1.max(new.1)), i32::from(old.0.min(new.0))),
                _ => (i32::from(new.1), HeightChange::FIRST_LOAD_LOWER),
            };
            let (x0, z0) = (pos.min_block_x(), pos.min_block_z());
            let change = HeightChange::new(BlockRect::new(x0, z0, x0 + 16, z0 + 16), higher, lower, pos.dim);
            self.mark_affected(store, &change);
        } else if result.changes.contains(ChangeKinds::SOLID) {
            self.mark_ring_soft(store, pos);
        }
    }

    // a block change that updated one column; old_height is the column height before the update
    pub fn on_column_submit(
        &mut self,
        store: &mut ChunkStore,
        pos: ChunkPos,
        column: (usize, usize),
        old_height: i16,
        new_height: i16,
        result: &SubmitResult,
    ) {
        // a biome-only change is journaled by the store but changes no baked pixel
        if !result.changes.intersects(ChangeKinds::PIXELS) {
            return;
        }
        self.mark_full(store, pos);
        if result.changes.contains(ChangeKinds::HEIGHT) {
            let x = pos.min_block_x() + column.0 as i32;
            let z = pos.min_block_z() + column.1 as i32;
            let higher = i32::from(old_height.max(new_height));
            let lower = i32::from(old_height.min(new_height));
            let change = HeightChange::new(BlockRect::new(x, z, x + 1, z + 1), higher, lower, pos.dim);
            self.mark_affected(store, &change);
        } else if result.changes.contains(ChangeKinds::SOLID) {
            self.mark_ring_soft(store, pos);
        }
    }

    // advances every region debounce; regions that fire become bake jobs
    pub fn tick(&mut self, store: &ChunkStore, jobs: &mut Vec<BakeJob>) {
        self.fired.clear();
        for (region, debounce) in self.regions.iter_mut() {
            if debounce.tick() {
                self.fired.push(*region);
            }
        }
        self.fired.sort_unstable();
        for i in 0..self.fired.len() {
            let region = self.fired[i];
            self.regions.remove(&region);
            store.dirty_in_region(region, &mut self.full, &mut self.soft);
            if self.full.len() > self.region_threshold {
                jobs.push(BakeJob::Region(region));
                continue;
            }
            jobs.extend(self.full.iter().map(|&pos| BakeJob::Chunk { pos, soft_only: false }));
            jobs.extend(self.soft.iter().map(|&pos| BakeJob::Chunk { pos, soft_only: true }));
        }
    }

    pub fn complete_chunk(
        &mut self,
        store: &mut ChunkStore,
        pos: ChunkPos,
        soft_only: bool,
        epochs: DirtyEpochs,
        mask_changed: bool,
    ) {
        if !soft_only && !store.clear_full_if(pos, epochs.full) {
            self.touch(pos.region());
        }
        store.clear_soft_if(pos, epochs.soft);
        if !soft_only && mask_changed {
            self.mark_ring_soft(store, pos);
        }
    }

    // `baked` holds the epochs snapshotted at bake start; `boundary_changed` are edge chunks whose mask changed
    pub fn complete_region(
        &mut self,
        store: &mut ChunkStore,
        region: RegionPos,
        baked: &[(ChunkPos, DirtyEpochs)],
        boundary_changed: &[ChunkPos],
    ) {
        let mut kept = false;
        for &(pos, epochs) in baked {
            if store.clear_full_if(pos, epochs.full) {
                store.clear_soft_if(pos, epochs.soft);
            } else {
                kept = true;
            }
        }
        if kept {
            self.touch(region);
        }
        for &pos in boundary_changed {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let neighbour = pos.offset(dx, dz);
                    if (dx != 0 || dz != 0) && !region.contains(&neighbour) {
                        self.mark_soft(store, neighbour);
                    }
                }
            }
        }
    }

    fn mark_affected(&mut self, store: &mut ChunkStore, change: &HeightChange) {
        self.tracer.trace(change, self.light, &*store, &mut self.affected);
        for i in 0..self.affected.len() {
            let chunk = self.affected[i];
            self.mark_full(store, chunk);
        }
    }

    fn mark_ring_soft(&mut self, store: &mut ChunkStore, center: ChunkPos) {
        for dz in -1..=1 {
            for dx in -1..=1 {
                if dx != 0 || dz != 0 {
                    self.mark_soft(store, center.offset(dx, dz));
                }
            }
        }
    }

    fn mark_full(&mut self, store: &mut ChunkStore, pos: ChunkPos) {
        if store.mark_full(pos) {
            self.touch(pos.region());
        }
    }

    fn mark_soft(&mut self, store: &mut ChunkStore, pos: ChunkPos) {
        if store.mark_soft(pos) {
            self.touch(pos.region());
        }
    }

    fn touch(&mut self, region: RegionPos) {
        self.regions.entry(region).and_modify(RegionDebounce::mark).or_default();
    }
}
