use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use minimap_core::config::Config;
use minimap_core::services::biome::BiomeResolver;
use minimap_core::services::cave::{CaveSurface, LayerSelector};
use minimap_core::services::dirty::DirtyTracker;
use minimap_core::services::export::AreaLayout;
use minimap_core::services::memory::ChunkStore;
use minimap_core::services::scan::{ScanOutcome, ScanScheduler};
use minimap_core::structs::bake::BakeJob;
use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface, ColumnSample, EvictedChunk};
use minimap_core::structs::export::ViewRegions;
use minimap_core::structs::layer::MapLayer;
use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_core::structs::tile::Tile;
use minimap_core::traits::BakeEnvironment;
use minimap_store::services::StoreService;
use minimap_store::structs::{JournalRecord, StoreCommand, StoreEvent, StoreSettings};

use crate::errors::RuntimeError;
use crate::services::{BakePool, ExportService, JournalOutbox};
use crate::structs::{BakeResult, BakedChunk, JournalAck, ProbeRequest, RuntimeStats, ScanPlan, Shared, SubmitStatus};

// everything that lives for one world visit: schedule, dirty marks, store, bake threads
pub struct WorldSession {
    shared: Arc<Shared>,
    store: StoreService,
    pool: BakePool,
    scheduler: ScanScheduler,
    tracker: DirtyTracker,
    outbox: JournalOutbox,
    biomes: BiomeResolver,
    deferred: Vec<(ChunkPos, Box<ChunkSurface>)>,
    results: Vec<BakeResult>,
    events: Vec<StoreEvent>,
    acks: Vec<JournalAck>,
    restores: Vec<(ChunkPos, Box<ChunkSurface>, bool)>,
    jobs: Vec<BakeJob>,
    evicted: Vec<EvictedChunk>,
    requested_tiles: HashSet<RegionPos>,
    requested_surfaces: HashSet<RegionPos>,
    requested_biomes: HashSet<RegionPos>,
    rebake_regions: HashSet<RegionPos>,
    // the map layer id: the dimension, or a cave band's layer id
    dim: i32,
    real_dim: i32,
    camera_height: i32,
    band_size: i32,
    rock: CaveSurface,
    // the band floor while a cave band is shown
    cave_floor: Option<i16>,
    tick: u64,
    scan_radius: i32,
    scan_limit: usize,
    player_chunk: ChunkPos,
    stats: RuntimeStats,
    exports: Arc<ExportService>,
    root: Option<PathBuf>,
    max_export_pixels: u32,
    // the dimension whose region list an explored-area export waits for
    area_export: Option<i32>,
}

impl WorldSession {
    pub const DEFERRED_CAPACITY: usize = 256;
    pub const REQUESTED_CAPACITY: usize = 512;
    const RESULTS_PER_TICK: usize = 16;
    const EVENTS_PER_TICK: usize = 256;
    const SHUTDOWN: Duration = Duration::from_secs(5);

    pub fn start(
        shared: Arc<Shared>,
        config: &Config,
        root: Option<PathBuf>,
        dim: i32,
        exports: Arc<ExportService>,
    ) -> Result<Self, RuntimeError> {
        let store = StoreService::start(root.clone(), StoreSettings::new(&config.store))?;
        let pool = BakePool::new(
            config.bake.worker_threads.max(0) as usize,
            config.bake.queue_capacity as usize,
            Arc::clone(&shared),
        )?;
        shared.world_epoch.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Self::open_biome_keys(&shared, &store, root.is_some());
        Ok(Self {
            store,
            pool,
            scheduler: ScanScheduler::new(config.terrain.rescan_interval_frames.max(1) as u64),
            tracker: DirtyTracker::new(shared.settings.light(), config.terrain.region_bake_threshold.max(1) as usize),
            outbox: JournalOutbox::new(config.store.write_queue_capacity.max(1) as usize * 4),
            biomes: BiomeResolver::new(),
            shared,
            deferred: Vec::with_capacity(Self::DEFERRED_CAPACITY),
            results: Vec::new(),
            events: Vec::new(),
            acks: Vec::new(),
            restores: Vec::new(),
            jobs: Vec::new(),
            evicted: Vec::new(),
            requested_tiles: HashSet::new(),
            requested_surfaces: HashSet::new(),
            requested_biomes: HashSet::new(),
            rebake_regions: HashSet::new(),
            dim,
            real_dim: dim,
            camera_height: LayerSelector::surface_height(dim),
            band_size: config.cave_mode.band_size.max(1),
            rock: CaveSurface::new(config.cave_mode.solid_color),
            cave_floor: None,
            tick: 0,
            scan_radius: config.terrain.scan_radius / 16,
            scan_limit: config.terrain.scan_max_chunks_per_frame.max(1) as usize,
            player_chunk: ChunkPos::new(0, 0, dim),
            stats: RuntimeStats::default(),
            exports,
            root,
            max_export_pixels: config.export.max_pixels.max(1) as u32,
            area_export: None,
        })
    }

    pub fn dim(&self) -> i32 {
        self.dim
    }

    // with a disk cache the store loads the world's table (or starts one); without, a new table is used
    fn open_biome_keys(shared: &Shared, store: &StoreService, cached: bool) {
        let mut keys = Shared::lock(&shared.biome_keys);
        keys.clear();
        if !cached || store.send(StoreCommand::LoadBiomeKeys).is_err() {
            keys.mark_loaded(Self::new_table_id(), Vec::new());
        }
    }

    // non-zero and different for every new table: the clock mixed with the process ID
    fn new_table_id() -> u64 {
        let nanos =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
        (nanos ^ (u64::from(std::process::id()) << 32)).max(1)
    }

    // each biome ID is answered once per world, so this line stays rare
    pub fn biome_name(&mut self, id: u16, name: &str) {
        if name.is_empty() {
            curia::warn!("the game has no name for a biome id", { id: id });
        }
        self.biomes.answer(id, name);
    }

    // the table's new names go to the store; a full queue keeps them for the next tick
    fn save_biome_keys(&mut self) {
        let mut keys = Shared::lock(&self.shared.biome_keys);
        let Some(names) = keys.take_dirty() else { return };
        let table_id = keys.table_id();
        if self.store.try_send(StoreCommand::WriteBiomeKeys { table_id, names }).is_err() {
            keys.mark_dirty();
        }
    }

    pub fn stats(&self) -> RuntimeStats {
        RuntimeStats {
            in_world: true,
            ticks: self.tick,
            bake_queue: self.pool.queued(),
            outbox: self.outbox.len(),
            deferred: self.deferred.len(),
            disk_written_bytes: self.store.bytes_written(),
            ..self.stats
        }
    }

    // a new layer (a dimension, or a cave band) starts with an empty chunk store and schedule
    pub fn change_layer(&mut self, layer: &MapLayer) {
        self.dim = layer.id;
        self.real_dim = layer.dim;
        self.camera_height = layer.camera_height;
        self.scan_radius = layer.scan_radius_chunks;
        self.cave_floor = layer.band.map(|band| (band * self.band_size) as i16);
        self.flush_evicted();
        if let Ok(mut chunks) = self.shared.chunks.write() {
            chunks.clear();
        }
        self.scheduler.clear();
        self.tracker.clear();
        self.deferred.clear();
        self.requested_surfaces.clear();
        self.requested_biomes.clear();
        self.rebake_regions.clear();
    }

    // folds the journal into the region files, then asks for the dimension's list of them
    pub fn request_area_export(&mut self) {
        if self.exports.is_busy() {
            Shared::lock(&self.shared.ui).show_message("An export is already running".to_string());
            return;
        }
        let compacted = self.store.try_send(StoreCommand::Compact).is_ok();
        if !compacted || self.store.try_send(StoreCommand::ListRegions { dim: self.dim }).is_err() {
            Shared::lock(&self.shared.ui).show_message("Export not started: the disk cache is busy".to_string());
            return;
        }
        self.area_export = Some(self.dim);
    }

    // the regions on screen, from the tile store or the region files, at the scale that fits max_pixels
    pub fn request_view_export(&mut self, view: ViewRegions) {
        let regions = view.regions();
        let message = match AreaLayout::new(&regions, self.max_export_pixels) {
            None => "The view is too large to export".to_string(),
            Some(layout) => match self.exports.export_view(layout, regions, self.root.clone(), view.layer) {
                Ok(()) => format!("Exporting {} x {} pixels", layout.width, layout.height),
                Err(message) => message,
            },
        };
        Shared::lock(&self.shared.ui).show_message(message);
    }

    // a missing table (no readable file) starts a new one, saved on the next tick
    fn on_biome_keys(&mut self, table: Option<(u64, Vec<String>)>) {
        let mut keys = Shared::lock(&self.shared.biome_keys);
        match table {
            Some((table_id, names)) => keys.mark_loaded(table_id, names),
            None => {
                keys.mark_loaded(Self::new_table_id(), Vec::new());
                keys.mark_dirty();
            }
        }
    }

    // the stored regions and those only in memory make the area
    fn on_region_list(&mut self, dim: i32, mut regions: Vec<RegionPos>) {
        if self.area_export.take() != Some(dim) || dim != self.dim {
            return;
        }
        let mut resident = Vec::new();
        Shared::lock(&self.shared.tiles).regions(dim, &mut resident);
        regions.extend(resident);
        regions.sort_by_key(|r| (r.z, r.x));
        regions.dedup();
        let message = match AreaLayout::new(&regions, self.max_export_pixels) {
            None if regions.is_empty() => "Nothing is explored here yet".to_string(),
            None => "The explored area is too large to export".to_string(),
            Some(layout) => match self.exports.export_area(layout, regions, self.root.clone(), dim) {
                Ok(()) => format!("Exporting {} x {} pixels", layout.width, layout.height),
                Err(message) => message,
            },
        };
        Shared::lock(&self.shared.ui).show_message(message);
    }

    pub fn tick(&mut self, player_chunk: ChunkPos, plan: &mut ScanPlan) {
        self.tick += 1;
        self.player_chunk = player_chunk;
        self.drain_store_events();
        self.pool.drain(&mut self.results, Self::RESULTS_PER_TICK);
        let shared = Arc::clone(&self.shared);
        if let Ok(mut chunks) = shared.chunks.try_write() {
            self.apply_pending(&mut chunks);
        }
        self.forward_tile_requests();
        self.forward_biome_requests();
        self.prefetch_surfaces();
        self.scheduler.update_center(player_chunk, self.scan_radius, self.tick);
        if let Ok(chunks) = shared.chunks.try_read() {
            self.scheduler.take_due(self.tick, self.scan_limit * 4, &*chunks, &mut plan.candidates);
            self.jobs.clear();
            self.tracker.tick(&chunks, &mut self.jobs);
        }
        for i in 0..self.jobs.len() {
            let job = self.jobs[i];
            if !self.pool.try_submit(job) {
                let region = match job {
                    BakeJob::Region(region) => region,
                    BakeJob::Chunk { pos, .. } => pos.region(),
                };
                self.tracker.requeue(region);
            }
        }
        self.save_biome_keys();
        self.outbox.flush(&self.store);
        self.biomes.take_requests(&mut plan.biome_requests);
        plan.limit = self.scan_limit;
        plan.camera_height = self.camera_height;
        plan.transparent_water = self.shared.settings.transparent_water;
    }

    pub fn submit_chunk(&mut self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe) {
        let outcome = match status {
            SubmitStatus::NotLoaded => ScanOutcome::NotLoaded,
            SubmitStatus::TopPlaceholder => ScanOutcome::TopPlaceholder,
            SubmitStatus::NotProbed => ScanOutcome::NotProbed,
            SubmitStatus::Loaded => {
                self.stats.chunks_submitted += 1;
                let mut prepared = probe.surface;
                self.biomes.fill(&probe.biome_ids, &mut prepared, &mut Shared::lock(&self.shared.biome_keys));
                if let Some(floor) = self.cave_floor {
                    self.rock.prepare_chunk(&mut prepared, floor);
                }
                let surface = &prepared;
                self.forget_hover_region(pos);
                let needs_retry = surface.any_skipped() || surface.any_retry();
                let shared = Arc::clone(&self.shared);
                if let Ok(mut chunks) = shared.chunks.try_write() {
                    self.apply_submit(&mut chunks, pos, surface);
                } else if self.deferred.len() < Self::DEFERRED_CAPACITY {
                    self.deferred.push((pos, Box::new(*surface)));
                } else {
                    self.scheduler.report(pos, ScanOutcome::NotProbed, self.tick);
                    return;
                }
                ScanOutcome::Scanned { needs_retry }
            }
        };
        self.scheduler.report(pos, outcome, self.tick);
    }

    // a block change only needs the one column when the chunk is already known
    // a block change in the real dimension asks for the column of the layer shown
    pub fn block_changed(&self, x: i32, z: i32, dim: i32) -> ProbeRequest {
        let pos = ChunkPos::from_block(x, z, self.dim);
        let known = self
            .shared
            .chunks
            .try_read()
            .is_ok_and(|chunks| chunks.surface(pos).is_some_and(ChunkSurface::has_base_data));
        if dim != self.real_dim || !known {
            return ProbeRequest::None;
        }
        ProbeRequest::Column { pos, index: ChunkSurface::index(x.rem_euclid(16) as usize, z.rem_euclid(16) as usize) }
    }

    pub fn submit_column(&mut self, pos: ChunkPos, index: usize, sample: ColumnSample) {
        let sample = match self.cave_floor {
            Some(floor) => self.rock.prepare_column(sample, floor),
            None => sample,
        };
        let shared = Arc::clone(&self.shared);
        let Ok(mut chunks) = shared.chunks.try_write() else { return };
        let Some(old_height) = chunks.surface(pos).map(|s| s.height[index]) else { return };
        let biome = self.biomes.key(sample.biome_id, &mut Shared::lock(&shared.biome_keys));
        self.forget_hover_region(pos);
        if let Some(result) = chunks.submit_column(pos, index, sample, biome) {
            let column = (index % 16, index / 16);
            self.tracker.on_column_submit(&mut chunks, pos, column, old_height, sample.height, &result);
        }
    }

    // journals every chunk that is still dirty, then stops the bake threads and the store
    pub fn finish(mut self) {
        let mut pending = Vec::new();
        self.pool.finish(Self::SHUTDOWN, |result| pending.push(result));
        let shared = Arc::clone(&self.shared);
        if let Ok(mut chunks) = shared.chunks.write() {
            for result in pending {
                self.apply_result(&mut chunks, result);
            }
            chunks.drain_journal_dirty(&mut self.evicted);
        }
        // the table goes first and is never dropped: the records below may use its newest keys
        let keys = {
            let mut table = Shared::lock(&self.shared.biome_keys);
            let table_id = table.table_id();
            table.take_dirty().map(|names| (table_id, names))
        };
        if let Some((table_id, names)) = keys
            && self.store.send(StoreCommand::WriteBiomeKeys { table_id, names }).is_err()
        {
            curia::warn!("the store stopped before the biome key table was saved");
        }
        let mut records = self.outbox.take_all();
        for evicted in std::mem::take(&mut self.evicted) {
            records.push(Box::new(JournalRecord::new(evicted.pos, self.shared.settings_hash, evicted.surface, None)));
        }
        for record in records {
            if self.store.send(StoreCommand::Append { record, token: StoreCommand::NO_ACK }).is_err() {
                curia::warn!("the store stopped before every chunk was journaled");
                break;
            }
        }
        let report = self.store.shutdown(Self::SHUTDOWN);
        if !report.finished {
            curia::warn!("the store did not finish in time", { timeout_ms: Self::SHUTDOWN.as_millis() as u64 });
        }
        self.shared.world_epoch.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn apply_pending(&mut self, chunks: &mut ChunkStore) {
        for (pos, surface) in std::mem::take(&mut self.deferred) {
            self.apply_submit(chunks, pos, &surface);
        }
        let restores = std::mem::take(&mut self.restores);
        for (pos, surface, baked) in &restores {
            let region = pos.region();
            let restored = chunks.restore(*pos, surface, self.player_chunk, &mut self.evicted);
            if restored && (!*baked || self.rebake_regions.contains(&region)) {
                chunks.mark_full(*pos);
                self.tracker.requeue(region);
            }
        }
        // a region's surfaces arrive in one answer, so once they are restored its rebake mark is spent
        for (pos, _, _) in &restores {
            self.rebake_regions.remove(&pos.region());
        }
        for result in std::mem::take(&mut self.results) {
            self.apply_result(chunks, result);
        }
        for ack in std::mem::take(&mut self.acks) {
            chunks.mark_journaled_if(ack.pos, ack.journal_version);
            if let Some((region, version)) = ack.tile {
                Shared::lock(&self.shared.tiles).mark_clean_if(region, version);
            }
        }
        self.flush_evicted();
    }

    fn apply_submit(&mut self, chunks: &mut ChunkStore, pos: ChunkPos, surface: &ChunkSurface) {
        let result = chunks.submit(pos, surface, self.tick, self.player_chunk, &mut self.evicted);
        self.tracker.on_chunk_submit(chunks, pos, &result);
    }

    fn apply_result(&mut self, chunks: &mut ChunkStore, result: BakeResult) {
        match result {
            BakeResult::Skipped => {}
            BakeResult::Chunk { chunk, soft_only, tile_version } => {
                self.stats.bakes_completed += 1;
                let mask_changed = chunks.store_mask(chunk.pos, &chunk.mask);
                self.tracker.complete_chunk(chunks, chunk.pos, soft_only, chunk.epochs, mask_changed && !soft_only);
                self.journal(chunks, *chunk, tile_version);
            }
            BakeResult::Region { region, chunks: baked, tile_version } => {
                self.stats.bakes_completed += baked.len() as u64;
                let mut epochs = Vec::with_capacity(baked.len());
                let mut boundary = Vec::new();
                for chunk in &baked {
                    let (lx, lz) = chunk.pos.local_in_region();
                    let edge = lx == 0 || lz == 0 || lx == 15 || lz == 15;
                    if chunks.store_mask(chunk.pos, &chunk.mask) && edge {
                        boundary.push(chunk.pos);
                    }
                    epochs.push((chunk.pos, chunk.epochs));
                }
                self.tracker.complete_region(chunks, region, &epochs, &boundary);
                for chunk in baked {
                    self.journal(chunks, chunk, tile_version);
                }
            }
        }
    }

    fn journal(&mut self, chunks: &ChunkStore, chunk: BakedChunk, tile_version: u64) {
        let Some(journal_version) = chunks.journal_version(chunk.pos) else { return };
        let ack = JournalAck { pos: chunk.pos, journal_version, tile: Some((chunk.pos.region(), tile_version)) };
        let record = JournalRecord::new(chunk.pos, self.shared.settings_hash, *chunk.surface, Some(chunk.pixels));
        self.outbox.push(record, ack);
    }

    fn flush_evicted(&mut self) {
        for evicted in std::mem::take(&mut self.evicted) {
            let ack = JournalAck { pos: evicted.pos, journal_version: u64::MAX, tile: None };
            self.outbox.push(JournalRecord::new(evicted.pos, self.shared.settings_hash, evicted.surface, None), ack);
        }
    }

    fn drain_store_events(&mut self) {
        self.events.clear();
        self.store.drain_events(&mut self.events, Self::EVENTS_PER_TICK);
        let mut evicted_tiles: Vec<(RegionPos, Tile)> = Vec::new();
        for event in std::mem::take(&mut self.events) {
            match event {
                StoreEvent::Appended { token, .. } => {
                    if let Some(ack) = self.outbox.acknowledge(token) {
                        self.acks.push(ack);
                    }
                }
                StoreEvent::TileLoaded { region, pixels, settings_hash } => {
                    self.stats.tiles_loaded += 1;
                    self.requested_tiles.remove(&region);
                    let mut tiles = Shared::lock(&self.shared.tiles);
                    if let Err(error) = tiles.insert_loaded(region, &pixels, settings_hash, &mut evicted_tiles) {
                        curia::warn!("tile not loaded", { x: region.x, z: region.z, layer: region.dim, error: error.to_string() });
                    }
                    if settings_hash != self.shared.settings_hash && self.near_player(region) {
                        self.rebake_regions.insert(region);
                        self.requested_surfaces.remove(&region);
                    }
                }
                StoreEvent::TileMissing(region) => {
                    self.requested_tiles.remove(&region);
                    if Shared::lock(&self.shared.tiles).peek(region).is_none() {
                        Shared::lock(&self.shared.known_empty).insert(region);
                    }
                }
                StoreEvent::SurfacesLoaded { chunks, .. } => self.restores.extend(chunks),
                StoreEvent::Failed(message) => curia::warn!("store command failed", { error: message }),
                // waypoints go through the game side's store, never the session's
                StoreEvent::WaypointsLoaded { .. } => {}
                StoreEvent::RegionList { dim, regions } => self.on_region_list(dim, regions),
                StoreEvent::BiomeKeysLoaded { table } => self.on_biome_keys(table),
                StoreEvent::BiomeKeysNotSaved => Shared::lock(&self.shared.biome_keys).mark_dirty(),
                StoreEvent::BiomesLoaded { region, biomes } => {
                    self.requested_biomes.remove(&region);
                    Shared::lock(&self.shared.hover_biomes).insert(region, *biomes);
                }
            }
        }
    }

    // a request may come for a tile already in memory: a bake that created the tile asks for the
    // stored pixels so they are merged under the new ones
    fn forward_tile_requests(&mut self) {
        // an answer the store dropped leaves its region here for good, so a full set starts again
        if self.requested_tiles.len() >= Self::REQUESTED_CAPACITY {
            self.requested_tiles.clear();
        }
        while let Ok(region) = self.shared.tile_requests.1.try_recv() {
            if self.requested_tiles.len() >= Self::REQUESTED_CAPACITY || self.requested_tiles.contains(&region) {
                continue;
            }
            if Shared::lock(&self.shared.known_empty).contains(region) {
                continue;
            }
            if self.store.try_send(StoreCommand::LoadTile(region)).is_ok() {
                self.requested_tiles.insert(region);
            }
        }
    }

    // regions the hover info rests on; a region already cached or asked for is not read again
    fn forward_biome_requests(&mut self) {
        if self.requested_biomes.len() >= Self::REQUESTED_CAPACITY {
            self.requested_biomes.clear();
        }
        while let Ok(region) = self.shared.biome_requests.1.try_recv() {
            if self.requested_biomes.contains(&region) || Shared::lock(&self.shared.hover_biomes).contains(region) {
                continue;
            }
            if self.store.try_send(StoreCommand::LoadBiomes(region)).is_ok() {
                self.requested_biomes.insert(region);
            }
        }
    }

    // a region explored after it was read for hover is read again on its next hover
    fn forget_hover_region(&self, pos: ChunkPos) {
        if let Ok(mut hover) = self.shared.hover_biomes.try_lock() {
            hover.forget(pos.region());
        }
    }

    // stored surfaces near the player make shadows at the edge of explored land right after a restart
    fn prefetch_surfaces(&mut self) {
        let reach = self.scan_radius + 1;
        let first = self.player_chunk.offset(-reach, -reach).region();
        let last = self.player_chunk.offset(reach, reach).region();
        if self.requested_surfaces.len() > Self::REQUESTED_CAPACITY {
            self.requested_surfaces.clear();
        }
        for rz in first.z..=last.z {
            for rx in first.x..=last.x {
                let region = RegionPos::new(rx, rz, self.dim);
                if self.requested_surfaces.contains(&region) {
                    continue;
                }
                if self.store.try_send(StoreCommand::LoadSurfaces(region)).is_ok() {
                    self.requested_surfaces.insert(region);
                }
            }
        }
    }

    fn near_player(&self, region: RegionPos) -> bool {
        let reach = self.scan_radius + 16;
        let centre = region.chunk(8, 8);
        region.dim == self.dim
            && (centre.x - self.player_chunk.x).abs() <= reach
            && (centre.z - self.player_chunk.z).abs() <= reach
    }
}
