use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use minimap_core::structs::biome::{BiomePalette, RegionBiomes};
use minimap_core::structs::pos::RegionPos;

use crate::errors::StoreError;
use crate::services::{BiomeKeyFile, CachePaths, Journal, LineFile, RegionFiles, WaypointFile};
use crate::structs::{JournalRecord, RegionSnapshot, StoreCommand, StoreEvent, StoreSettings};

// runs on the store thread: owns the journals and region files, answers commands with events
pub(crate) struct StoreWorker {
    root: Option<PathBuf>,
    files: Option<RegionFiles>,
    // the world's biome key table; appended records are stamped with it
    table_id: u64,
    journals: HashMap<i32, Journal>,
    settings: StoreSettings,
    commands: Receiver<StoreCommand>,
    events: Sender<StoreEvent>,
    stop: Arc<AtomicBool>,
    written: Arc<AtomicU64>,
    records: Vec<JournalRecord>,
    dirty_since_sync: bool,
    last_sync: Instant,
    last_activity: Instant,
}

impl StoreWorker {
    const POLL: Duration = Duration::from_millis(50);
    const EVENT_TIMEOUT: Duration = Duration::from_millis(100);

    pub(crate) fn new(
        root: Option<PathBuf>,
        settings: StoreSettings,
        commands: Receiver<StoreCommand>,
        events: Sender<StoreEvent>,
        stop: Arc<AtomicBool>,
        written: Arc<AtomicU64>,
    ) -> Self {
        let now = Instant::now();
        Self {
            files: root.clone().map(RegionFiles::new),
            table_id: 0,
            root,
            journals: HashMap::new(),
            settings,
            commands,
            events,
            stop,
            written,
            records: Vec::with_capacity(256),
            dirty_since_sync: false,
            last_sync: now,
            last_activity: now,
        }
    }

    pub(crate) fn run(mut self) {
        loop {
            match self.commands.recv_timeout(Self::POLL) {
                Ok(command) => {
                    self.last_activity = Instant::now();
                    self.handle(command);
                }
                Err(RecvTimeoutError::Timeout) => {
                    if self.stop.load(Ordering::SeqCst) {
                        break;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
            self.housekeeping();
            self.publish_written();
        }
        self.finish();
        self.publish_written();
    }

    fn handle(&mut self, command: StoreCommand) {
        let result = match command {
            StoreCommand::Append { mut record, token } => self.append(&mut record).map(|()| {
                if token != StoreCommand::NO_ACK {
                    self.emit(StoreEvent::Appended { pos: record.pos, token });
                }
            }),
            StoreCommand::LoadTile(region) => self.load_tile(region),
            StoreCommand::LoadSurfaces(region) => self.load_surfaces(region),
            StoreCommand::LoadBiomes(region) => self.load_biomes(region),
            StoreCommand::LoadBiomeKeys => self.load_biome_keys(),
            StoreCommand::WriteBiomeKeys { table_id, names } => {
                self.write_biome_keys(table_id, &names).inspect_err(|_| {
                    self.emit(StoreEvent::BiomeKeysNotSaved);
                })
            }
            StoreCommand::AppendLine { path, line, truncate } => LineFile::append(&path, &line, truncate),
            StoreCommand::Compact => self.compact_all(),
            StoreCommand::LoadWaypoints { path } => WaypointFile::load(&path).map(|waypoints| {
                self.emit(StoreEvent::WaypointsLoaded { path, waypoints });
            }),
            StoreCommand::WriteWaypoints { path, waypoints } => WaypointFile::save(&path, &waypoints),
            StoreCommand::ListRegions { dim } => self.list_regions(dim),
        };
        if let Err(error) = result {
            curia::error!("store command failed", { error: error.to_string() });
            self.emit(StoreEvent::Failed(error.to_string()));
        }
    }

    fn append(&mut self, record: &mut JournalRecord) -> Result<(), StoreError> {
        record.table_id = self.table_id;
        let Some(journal) = self.journal(record.pos.dim)? else { return Ok(()) };
        journal.append(record)?;
        self.dirty_since_sync = true;
        Ok(())
    }

    fn snapshot(&mut self, region: RegionPos) -> Result<Option<RegionSnapshot>, StoreError> {
        let Some(files) = &self.files else { return Ok(None) };
        let stored = files.read(region)?;
        let mut records = std::mem::take(&mut self.records);
        if let Some(journal) = self.journal(region.dim)? {
            journal.records_for_region(region, &mut records)?;
        }
        let snapshot = match (stored, records.is_empty()) {
            (None, true) => None,
            (stored, _) => {
                let first_hash = records.first().map_or(0, |r| r.settings_hash);
                let table_id = self.table_id;
                let mut snapshot = stored.unwrap_or_else(|| {
                    let mut snapshot = RegionSnapshot::new(region, first_hash);
                    snapshot.table_id = table_id;
                    snapshot
                });
                for record in &records {
                    snapshot.apply(record);
                }
                Some(snapshot)
            }
        };
        self.records = records;
        Ok(snapshot.filter(|s| !s.is_empty()))
    }

    fn load_tile(&mut self, region: RegionPos) -> Result<(), StoreError> {
        let event = match self.snapshot(region)? {
            Some(snapshot) => {
                StoreEvent::TileLoaded { region, pixels: snapshot.tile, settings_hash: snapshot.settings_hash }
            }
            None => StoreEvent::TileMissing(region),
        };
        self.emit(event);
        Ok(())
    }

    fn load_surfaces(&mut self, region: RegionPos) -> Result<(), StoreError> {
        let chunks = match self.snapshot(region)? {
            Some(snapshot) => {
                let baked: Vec<bool> = (0..256).map(|i| snapshot.chunk_has_pixels(i % 16, i / 16)).collect();
                snapshot
                    .surfaces
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, s)| s.map(|s| (region.chunk(i % 16, i / 16), s, baked[i])))
                    .collect()
            }
            None => Vec::new(),
        };
        self.emit(StoreEvent::SurfacesLoaded { region, chunks });
        Ok(())
    }

    // the stored palettes with the journal laid over them; a record from another table has none
    fn load_biomes(&mut self, region: RegionPos) -> Result<(), StoreError> {
        let mut biomes = match &self.files {
            Some(files) => files.read_biomes(region)?.unwrap_or_else(RegionBiomes::empty),
            None => RegionBiomes::empty(),
        };
        let mut records = std::mem::take(&mut self.records);
        if let Some(journal) = self.journal(region.dim)? {
            journal.records_for_region(region, &mut records)?;
        }
        for record in &records {
            let palette = if record.table_id == self.table_id { record.surface.biome } else { BiomePalette::empty() };
            biomes.set(record.pos.local_in_region(), palette);
        }
        self.records = records;
        self.emit(StoreEvent::BiomesLoaded { region, biomes: Box::new(biomes) });
        Ok(())
    }

    // a world with no readable table starts a new one, saved at once, so its ID never changes
    fn load_biome_keys(&mut self) -> Result<(), StoreError> {
        let Some(root) = &self.root else {
            self.emit(StoreEvent::BiomeKeysLoaded { table: None });
            return Ok(());
        };
        let path = CachePaths::biome_keys_file(root);
        let loaded = BiomeKeyFile::load(&path).unwrap_or_else(|error| {
            curia::warn!("biome key file not read; a new table is started", { path: path.display().to_string(), error: error.to_string() });
            None
        });
        let table = match loaded {
            Some(table) => table,
            None => {
                let table_id = Self::new_table_id();
                if let Err(error) = BiomeKeyFile::save(&path, table_id, &[]) {
                    curia::warn!("biome key file not written; it is tried again on the next change", {
                        path: path.display().to_string(),
                        error: error.to_string(),
                    });
                }
                (table_id, Vec::new())
            }
        };
        self.use_table(table.0);
        self.emit(StoreEvent::BiomeKeysLoaded { table: Some(table) });
        Ok(())
    }

    fn write_biome_keys(&mut self, table_id: u64, names: &[String]) -> Result<(), StoreError> {
        self.use_table(table_id);
        let Some(root) = &self.root else { return Ok(()) };
        BiomeKeyFile::save(&CachePaths::biome_keys_file(root), table_id, names)
    }

    fn use_table(&mut self, table_id: u64) {
        self.table_id = table_id;
        if let Some(files) = &mut self.files {
            files.set_table_id(table_id);
        }
    }

    // non-zero and different for every new table: the clock mixed with the process ID
    fn new_table_id() -> u64 {
        let nanos =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
        (nanos ^ (u64::from(std::process::id()) << 32)).max(1)
    }

    fn list_regions(&mut self, dim: i32) -> Result<(), StoreError> {
        let regions = match &self.files {
            Some(files) => files.list(dim)?,
            None => Vec::new(),
        };
        self.emit(StoreEvent::RegionList { dim, regions });
        Ok(())
    }

    fn compact_all(&mut self) -> Result<(), StoreError> {
        let Some(files) = &self.files else { return Ok(()) };
        for journal in self.journals.values_mut() {
            journal.compact(files)?;
        }
        Ok(())
    }

    fn housekeeping(&mut self) {
        let now = Instant::now();
        if self.dirty_since_sync && now.duration_since(self.last_sync) >= self.settings.sync_interval {
            self.sync_all();
        }
        let oversized = self.journals.values().any(Journal::needs_compaction);
        let idle = now.duration_since(self.last_activity) >= self.settings.idle_compaction
            && self.journals.values().any(|j| j.indexed_len() > 0);
        if (oversized || idle)
            && let Err(error) = self.compact_all()
        {
            curia::error!("journal compaction failed", { error: error.to_string() });
        }
    }

    fn sync_all(&mut self) {
        for journal in self.journals.values() {
            if let Err(error) = journal.sync() {
                curia::error!("journal sync failed", { error: error.to_string() });
            }
        }
        self.dirty_since_sync = false;
        self.last_sync = Instant::now();
    }

    // drains what is already queued, then syncs and compacts
    fn finish(&mut self) {
        while let Ok(command) = self.commands.try_recv() {
            self.handle(command);
        }
        self.sync_all();
        if let Err(error) = self.compact_all() {
            curia::error!("journal compaction at shutdown failed", { error: error.to_string() });
        }
    }

    fn journal(&mut self, dim: i32) -> Result<Option<&mut Journal>, StoreError> {
        let Some(root) = &self.root else { return Ok(None) };
        if !self.journals.contains_key(&dim) {
            let journal = Journal::open(&CachePaths::journal_dir(root, dim), self.settings.journal_max_bytes)?;
            self.journals.insert(dim, journal);
        }
        Ok(self.journals.get_mut(&dim))
    }

    fn publish_written(&self) {
        let total = self.journals.values().map(Journal::bytes_written).sum();
        self.written.store(total, Ordering::Relaxed);
    }

    fn emit(&self, event: StoreEvent) {
        if self.events.send_timeout(event, Self::EVENT_TIMEOUT).is_err() {
            curia::warn!("store event dropped: the event queue is full");
        }
    }
}
