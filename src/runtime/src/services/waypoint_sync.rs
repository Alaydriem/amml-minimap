use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use minimap_core::errors::WaypointError;
use minimap_core::structs::waypoint::{Waypoint, WaypointSource};
use minimap_store::services::StoreService;
use minimap_store::structs::StoreCommand;

use crate::structs::{Shared, TickInput};

// keeps the current world's waypoints in step with its waypoints.json, through the game side's store
pub struct WaypointSync {
    shared: Arc<Shared>,
    file: Option<PathBuf>,
    // the file has been read, so a save cannot overwrite waypoints that are still only on disk
    loaded: bool,
    changed_at: Option<Instant>,
    was_dead: bool,
}

impl WaypointSync {
    pub const SAVE_DELAY: Duration = Duration::from_secs(2);

    pub fn new(shared: Arc<Shared>) -> Self {
        Self { shared, file: None, loaded: false, changed_at: None, was_dead: false }
    }

    // the current world's file has been read; until then nothing is saved
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    // a world was entered: its file is read on the store thread
    pub fn open(&mut self, path: PathBuf, store: &StoreService) {
        if store.try_send(StoreCommand::LoadWaypoints { path: path.clone() }).is_err() {
            curia::warn!("waypoints not loaded: the store queue is full");
        }
        self.file = Some(path);
        self.loaded = false;
        self.changed_at = None;
        self.was_dead = false;
    }

    // the world is left: a pending change is saved, then the list is emptied
    pub fn close(&mut self, store: &StoreService) {
        if self.file.is_some() && (self.changed_at.is_some() || Shared::lock(&self.shared.waypoints).take_dirty()) {
            self.save(store);
        }
        Shared::lock(&self.shared.waypoints).clear();
        self.file = None;
        self.changed_at = None;
    }

    // an answer for an earlier world is ignored
    pub fn on_loaded(&mut self, path: &Path, waypoints: Vec<Waypoint>) {
        if self.file.as_deref() != Some(path) {
            return;
        }
        let count = waypoints.len();
        let dropped = Shared::lock(&self.shared.waypoints).merge_loaded(waypoints);
        if dropped > 0 {
            curia::warn!("waypoints were dropped on load", { dropped: dropped, count: count, path: path.display().to_string() });
        }
        self.loaded = true;
    }

    // the death edge sets the death point; changes are saved SAVE_DELAY after the last one
    pub fn tick(&mut self, input: &TickInput, store: &StoreService) {
        if self.file.is_none() {
            return;
        }
        let dead = input.player.present && input.dead;
        if dead && !self.was_dead {
            let player = input.player;
            Shared::lock(&self.shared.waypoints).set_death(player.dim, player.block(), Self::now());
        }
        self.was_dead = dead;
        if Shared::lock(&self.shared.waypoints).take_dirty() {
            self.changed_at = Some(Instant::now());
        }
        if self.changed_at.is_some_and(|at| at.elapsed() >= Self::SAVE_DELAY) {
            self.save(store);
        }
    }

    // the hotkey: "Waypoint N" at the player's block
    pub fn add_at_player(&mut self) -> Result<(), WaypointError> {
        if self.file.is_none() {
            return Ok(());
        }
        let player = *Shared::lock(&self.shared.player);
        if !player.present {
            return Ok(());
        }
        let added = Shared::lock(&self.shared.waypoints).add(
            "",
            player.dim,
            player.block(),
            WaypointSource::Hotkey,
            Self::now(),
        );
        added.map(|_| ())
    }

    // a full queue or a file not yet read keeps the change pending for the next try
    fn save(&mut self, store: &StoreService) {
        let Some(path) = self.file.clone() else { return };
        if !self.loaded {
            curia::warn!("waypoints not saved yet: the file has not been read", { path: path.display().to_string() });
            self.changed_at = Some(Instant::now());
            return;
        }
        let waypoints = Shared::lock(&self.shared.waypoints).waypoints().to_vec();
        match store.try_send(StoreCommand::WriteWaypoints { path, waypoints }) {
            Ok(()) => self.changed_at = None,
            Err(_) => {
                curia::warn!("waypoints not saved yet: the store queue is full");
                self.changed_at = Some(Instant::now());
            }
        }
    }

    pub fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
    }
}
