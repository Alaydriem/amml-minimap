use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use minimap_core::services::bake::{BakeReach, ChunkBaker, RegionBaker, SnapshotEnvironment};
use minimap_core::structs::bake::{BakeJob, BakeOutput, RegionBakeOutput};
use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_core::traits::BakeEnvironment;

use crate::structs::{BakeResult, BakedChunk, Shared};

// one bake thread: reads chunks under the shared read lock, writes pixels into tiles
pub(crate) struct BakeWorker {
    shared: Arc<Shared>,
    jobs: Receiver<BakeJob>,
    results: Sender<BakeResult>,
    stop: Arc<AtomicBool>,
    chunk: ChunkBaker,
    region: RegionBaker,
    chunk_out: BakeOutput,
    region_out: Box<RegionBakeOutput>,
    snapshot: SnapshotEnvironment,
    reach: (i32, i32, i32, i32),
}

impl BakeWorker {
    const POLL: Duration = Duration::from_millis(50);

    pub(crate) fn new(
        shared: Arc<Shared>,
        jobs: Receiver<BakeJob>,
        results: Sender<BakeResult>,
        stop: Arc<AtomicBool>,
    ) -> Self {
        let reach = BakeReach::offsets(shared.settings.light());
        Self {
            shared,
            jobs,
            results,
            stop,
            chunk: ChunkBaker::new(),
            region: RegionBaker::new(),
            chunk_out: BakeOutput::new(),
            region_out: Box::new(RegionBakeOutput::new()),
            snapshot: SnapshotEnvironment::new(),
            reach,
        }
    }

    pub(crate) fn run(mut self) {
        while !self.stop.load(Ordering::SeqCst) {
            let job = match self.jobs.recv_timeout(Self::POLL) {
                Ok(job) => job,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            };
            let started = std::time::Instant::now();
            let result = match job {
                BakeJob::Chunk { pos, soft_only } => self.bake_chunk(pos, soft_only),
                BakeJob::Region(region) => self.bake_region(region),
            };
            if !matches!(result, BakeResult::Skipped) {
                self.shared.bake_timing.record(started.elapsed());
            }
            if !self.deliver(result) {
                return;
            }
        }
    }

    // offers the result until it is taken; a stopping pool still drains results, so none is lost
    fn deliver(&self, result: BakeResult) -> bool {
        let mut pending = result;
        loop {
            match self.results.send_timeout(pending, Self::POLL) {
                Ok(()) => return true,
                Err(crossbeam_channel::SendTimeoutError::Timeout(back)) => pending = back,
                Err(crossbeam_channel::SendTimeoutError::Disconnected(_)) => return false,
            }
        }
    }

    fn bake_chunk(&mut self, pos: ChunkPos, soft_only: bool) -> BakeResult {
        let settings = self.shared.settings;
        let (dx0, dx1, dz0, dz1) = self.reach;
        let (surface, flags) = {
            let chunks = self.shared.chunks.read().unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(surface) = chunks.surface(pos).copied() else { return BakeResult::Skipped };
            let Some(flags) = chunks.dirty(pos) else { return BakeResult::Skipped };
            if soft_only && flags.is_full() {
                return BakeResult::Skipped;
            }
            self.snapshot.clear();
            self.snapshot.capture_area(&*chunks, (pos.x + dx0, pos.x + dx1), (pos.z + dz0, pos.z + dz1), pos.dim);
            (surface, flags)
        };
        match self.snapshot.mask(pos).cloned() {
            Some(stored) if soft_only => {
                self.chunk.bake_soft(pos, &surface, &stored, &self.snapshot, &settings, &mut self.chunk_out);
            }
            _ => self.chunk.bake(pos, &surface, &self.snapshot, &settings, &mut self.chunk_out),
        }
        let baked = BakedChunk {
            pos,
            epochs: flags.epochs(),
            mask: self.chunk_out.mask.clone(),
            pixels: self.chunk_out.pixels,
            surface: Box::new(surface),
        };
        let tile_version = self.write_tiles(pos.region(), std::slice::from_ref(&baked));
        BakeResult::Chunk { chunk: Box::new(baked), soft_only, tile_version }
    }

    fn bake_region(&mut self, region: RegionPos) -> BakeResult {
        let settings = self.shared.settings;
        let (dx0, dx1, dz0, dz1) = self.reach;
        let origin = region.chunk(0, 0);
        let mut epochs = Vec::with_capacity(256);
        {
            let chunks = self.shared.chunks.read().unwrap_or_else(std::sync::PoisonError::into_inner);
            self.snapshot.clear();
            let x = (origin.x + dx0, origin.x + 15 + dx1);
            let z = (origin.z + dz0, origin.z + 15 + dz1);
            self.snapshot.capture_area(&*chunks, x, z, region.dim);
            for i in 0..256 {
                let pos = region.chunk(i % 16, i / 16);
                if let Some(flags) = chunks.dirty(pos) {
                    epochs.push((pos, flags.epochs()));
                }
            }
        }
        self.region.bake(region, &self.snapshot, &settings, &mut self.region_out);
        let mut baked = Vec::with_capacity(epochs.len());
        for (pos, epochs) in epochs {
            let (lx, lz) = pos.local_in_region();
            let i = lz * 16 + lx;
            let Some(surface) = self.snapshot.surface(pos) else { continue };
            if !self.region_out.present[i] {
                continue;
            }
            baked.push(BakedChunk {
                pos,
                epochs,
                mask: self.region_out.masks[i].clone(),
                pixels: self.region_out.pixels[i],
                surface: Box::new(*surface),
            });
        }
        let tile_version = self.write_tiles(region, &baked);
        BakeResult::Region { region, chunks: baked, tile_version }
    }

    // dirty tiles that the store pushes out are already journaled, so they can be dropped
    fn write_tiles(&self, region: RegionPos, baked: &[BakedChunk]) -> u64 {
        let mut evicted = Vec::new();
        let mut tiles = Shared::lock(&self.shared.tiles);
        let mut created = false;
        for chunk in baked {
            created |= tiles.write_chunk(chunk.pos, &chunk.pixels, self.shared.settings_hash, &mut evicted);
        }
        let version = tiles.peek(region).map_or(0, |t| t.version());
        drop(tiles);
        if created {
            Shared::lock(&self.shared.known_empty).remove(region);
            let _ = self.shared.tile_requests.0.try_send(region);
        }
        version
    }
}
