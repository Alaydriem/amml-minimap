use std::sync::Arc;
use std::time::Duration;

use minimap_core::config::{Config, ConfigMerge};
use minimap_core::errors::WaypointError;
use minimap_core::services::cave::LayerSelector;
use minimap_core::services::input::InputController;
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::input::{KeyResult, KeybindInput};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::settings::{SettingValue, SettingWrite};
use minimap_store::services::{CachePaths, StoreService};
use minimap_store::structs::{StoreCommand, StoreEvent, StoreSettings, WorldIdentity};

use crate::errors::RuntimeError;
use crate::services::{ExportService, WaypointSync, WorldSession};
use crate::structs::{
    CursorAction, PendingSettings, ProbeRequest, RuntimePaths, RuntimeStats, ScanPlan, Shared, SubmitStatus, TickInput,
};

// everything the game thread calls: world lifecycle, ticks, probes, keys
pub struct GameSide {
    shared: Arc<Shared>,
    config: Config,
    paths: RuntimePaths,
    input: InputController,
    session: Option<WorldSession>,
    config_store: StoreService,
    waypoints: WaypointSync,
    store_events: Vec<StoreEvent>,
    exports: Arc<ExportService>,
    layers: LayerSelector,
    // in-play changes waiting to be saved as manager settings
    pending: PendingSettings,
    // the manager's latest config; its world settings apply when the next world starts
    next_world: Option<Config>,
}

impl GameSide {
    const SHUTDOWN: Duration = Duration::from_secs(5);
    const MAX_STORE_EVENTS: usize = 16;

    pub fn new(
        shared: Arc<Shared>,
        config: Config,
        paths: RuntimePaths,
        exports: Arc<ExportService>,
    ) -> Result<Self, RuntimeError> {
        let config_store = StoreService::start(None, StoreSettings::new(&config.store))?;
        let game = Self {
            input: InputController::new(&config),
            waypoints: WaypointSync::new(Arc::clone(&shared)),
            store_events: Vec::with_capacity(Self::MAX_STORE_EVENTS),
            exports,
            layers: LayerSelector::new(&config),
            pending: PendingSettings::new(),
            next_world: None,
            shared,
            config,
            paths,
            session: None,
            config_store,
        };
        Ok(game)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn waypoints_loaded(&self) -> bool {
        self.waypoints.is_loaded()
    }

    // takes the live part of a changed config at once and keeps the rest for the next world; returns the
    // config now in use
    pub fn apply_settings(&mut self, config: Config) -> Config {
        self.config = ConfigMerge::live(&self.config, &config);
        self.input = InputController::new(&self.config);
        self.next_world = Some(config);
        self.config.clone()
    }

    pub fn take_setting_writes(&mut self, out: &mut Vec<SettingWrite>) {
        self.pending.drain(out);
    }

    pub fn enter_world(&mut self, identity: &WorldIdentity, dim: i32) -> Result<(), RuntimeError> {
        self.leave_world();
        if let Some(next) = self.next_world.take() {
            self.config = ConfigMerge::world(&self.config, &next);
            self.input = InputController::new(&self.config);
            self.layers = LayerSelector::new(&self.config);
        }
        if self.config.waypoints.enabled {
            self.waypoints.open(CachePaths::waypoints_file(identity, &self.paths.data_dir), &self.config_store);
        }
        let root = self.config.terrain.enable_disk_cache.then(|| CachePaths::root(identity, &self.paths.data_dir));
        if let Some(root) = &root {
            match CachePaths::adopt_legacy(root) {
                Ok(true) => curia::info!("the map cache moved to its new folder name"),
                Ok(false) => {}
                Err(error) => {
                    curia::warn!("the old map cache could not be moved; a new cache starts", { error: error.to_string() })
                }
            }
        }
        self.exports.enter_world(&identity.label());
        self.layers.reset();
        self.shared.layer.store(dim, std::sync::atomic::Ordering::Relaxed);
        let exports = Arc::clone(&self.exports);
        self.session = Some(WorldSession::start(Arc::clone(&self.shared), &self.config, root, dim, exports)?);
        Ok(())
    }

    pub fn leave_world(&mut self) {
        if let Some(session) = self.session.take() {
            session.finish();
        }
        self.waypoints.close(&self.config_store);
        self.exports.leave_world();
        Shared::lock(&self.shared.ui).reset();
        if let Ok(mut chunks) = self.shared.chunks.write() {
            chunks.clear();
        }
        Shared::lock(&self.shared.tiles).clear();
        Shared::lock(&self.shared.known_empty).clear();
        Shared::lock(&self.shared.biome_keys).clear();
        Shared::lock(&self.shared.hover_biomes).clear();
        Shared::lock(&self.shared.player).present = false;
        let mut view = Shared::lock(&self.shared.view);
        view.world_map_open = false;
        view.anchor = None;
    }

    // the overlay switches of the tools block are saved like the coordinates switch
    fn sync_view_config(&mut self) {
        let (grid, slime) = {
            let view = Shared::lock(&self.shared.view);
            (view.show_chunk_grid, view.show_slime_chunks)
        };
        let world_map = &mut self.config.world_map;
        if world_map.show_chunk_grid != grid {
            world_map.show_chunk_grid = grid;
            self.pending.set(PendingSettings::CHUNK_GRID, SettingValue::Bool(grid));
        }
        if world_map.show_slime_chunks != slime {
            world_map.show_slime_chunks = slime;
            self.pending.set(PendingSettings::SLIME_CHUNKS, SettingValue::Bool(slime));
        }
    }

    pub fn tick(&mut self, input: &TickInput, plan: &mut ScanPlan) {
        *Shared::lock(&self.shared.player) = input.player;
        plan.candidates.clear();
        plan.cursor = self.cursor_action(input);
        self.sync_view_config();
        self.drain_store_events();
        self.waypoints.tick(input, &self.config_store);
        let Some(session) = self.session.as_mut() else { return };
        if !input.player.present {
            return;
        }
        let layer = self.layers.select(input.player.dim, input.player.block().1);
        if session.dim() != layer.id {
            session.change_layer(&layer);
            self.shared.layer.store(layer.id, std::sync::atomic::Ordering::Relaxed);
        }
        if self.shared.area_export.swap(false, std::sync::atomic::Ordering::SeqCst) {
            session.request_area_export();
        }
        if let Some(view) = Shared::lock(&self.shared.view_export).take() {
            session.request_view_export(view);
        }
        if self.config.terrain.enable {
            let (bx, _, bz) = input.player.block();
            session.tick(ChunkPos::from_block(bx, bz, layer.id), plan);
        }
    }

    pub fn submit_chunk(&mut self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe) {
        if let Some(session) = self.session.as_mut() {
            session.submit_chunk(pos, status, probe);
        }
    }

    pub fn biome_name(&mut self, id: u16, name: &str) {
        if let Some(session) = self.session.as_mut() {
            session.biome_name(id, name);
        }
    }

    pub fn block_changed(&self, x: i32, z: i32, dim: i32) -> ProbeRequest {
        self.session.as_ref().map_or(ProbeRequest::None, |s| s.block_changed(x, z, dim))
    }

    pub fn submit_column(&mut self, pos: ChunkPos, index: usize, sample: ColumnSample) {
        if let Some(session) = self.session.as_mut() {
            session.submit_column(pos, index, sample);
        }
    }

    pub fn keybind(&mut self, input: KeybindInput) -> KeyResult {
        // a world map text field has the keyboard: the key is a typed letter, not a keybind
        if self.shared.typing.load(std::sync::atomic::Ordering::Relaxed)
            && Shared::lock(&self.shared.view).world_map_open
        {
            return KeyResult::default();
        }
        let (result, radius, coordinates, closed) = {
            let mut view = Shared::lock(&self.shared.view);
            let was_open = view.world_map_open;
            let result = self.input.on_keybind(&mut view, input);
            (result, view.minimap_radius, view.show_coordinates, was_open && !view.world_map_open)
        };
        if result.coordinates_changed {
            self.config.mini_map.show_coordinates = coordinates;
            self.config.world_map.show_coordinates = coordinates;
            self.pending.set(PendingSettings::MINI_MAP_COORDINATES, SettingValue::Bool(coordinates));
            self.pending.set(PendingSettings::WORLD_MAP_COORDINATES, SettingValue::Bool(coordinates));
            let state = if coordinates { "shown" } else { "hidden" };
            Shared::lock(&self.shared.ui).show_done(format!("Coordinates {state}"));
        }
        if result.radius_changed {
            self.config.mini_map.radius_chunks = radius;
            self.pending.set(PendingSettings::RADIUS, SettingValue::Int(i64::from(radius)));
        }
        if result.add_waypoint
            && let Err(WaypointError::Full(cap)) = self.waypoints.add_at_player()
        {
            Shared::lock(&self.shared.ui).show_message(format!("This world already has {cap} waypoints"));
        }
        if result.toggle_cave {
            let state = if self.layers.toggle() { "on" } else { "off" };
            Shared::lock(&self.shared.ui).show_done(format!("Cave view {state}"));
        }
        if closed {
            Shared::lock(&self.shared.ui).close();
        }
        result
    }

    pub fn stats(&self) -> RuntimeStats {
        self.session.as_ref().map_or_else(RuntimeStats::default, WorldSession::stats)
    }

    pub fn shutdown(&mut self) {
        self.leave_world();
        self.config_store.shutdown(Self::SHUTDOWN);
    }

    // answers from the game side's store: waypoint files; telemetry and waypoint write failures
    fn drain_store_events(&mut self) {
        self.config_store.drain_events(&mut self.store_events, Self::MAX_STORE_EVENTS);
        for event in self.store_events.drain(..) {
            match event {
                StoreEvent::WaypointsLoaded { path, waypoints } => self.waypoints.on_loaded(&path, waypoints),
                StoreEvent::Failed(message) => curia::warn!("store command failed", { error: message }),
                _ => {}
            }
        }
    }

    // while the game holds the mouse, the cursor is kept at the client centre; out of a world it is free
    fn cursor_action(&self, input: &TickInput) -> CursorAction {
        if !input.player.present {
            return CursorAction::Release;
        }
        let map_open = Shared::lock(&self.shared.view).world_map_open;
        match (map_open, input.foreground, input.cursor_hidden) {
            (true, _, _) | (false, false, _) => CursorAction::None,
            (false, true, true) => CursorAction::ClipToCentre,
            (false, true, false) => CursorAction::Release,
        }
    }

    // telemetry lines go through the store thread too; a full queue drops the line
    pub fn append_line(&self, path: &std::path::Path, line: String, truncate: bool) {
        let command = StoreCommand::AppendLine { path: path.to_path_buf(), line, truncate };
        if self.config_store.try_send(command).is_err() {
            curia::warn!("telemetry line dropped: the store queue is full");
        }
    }
}
