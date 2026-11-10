use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use minimap_core::config::Config;
use minimap_core::services::biome::{BiomeColors, HoverBiomeCache};
use minimap_core::services::input::InputController;
use minimap_core::services::memory::{ChunkStore, KnownEmptySet, TileStore};
use minimap_core::services::ui::MapUi;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::Rgba;
use minimap_core::structs::bake::BakeSettings;
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::draw::{MapLayer, TextureGrant};
use minimap_core::structs::input::{KeyResult, KeybindInput};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::settings::SettingWrite;
use minimap_core::structs::view::{ViewFlags, ViewState};
use minimap_core::structs::waypoint::Waypoint;
use minimap_store::structs::WorldIdentity;

use crate::errors::RuntimeError;
use crate::services::{CaptureSink, CaptureWriter, ExportService, GameSide, RenderSide, TelemetryRecorder};
use crate::structs::{
    CaptureEvent, CaptureHeader, CursorAction, LayerInput, LayerPaint, LayerReply, ProbeRequest, RuntimePaths,
    RuntimeStats, ScanPlan, Shared, SubmitStatus, TelemetryRow, TickInput,
};

// the whole mod behind one handle; game-thread calls and render-thread calls take separate locks
pub struct Runtime {
    shared: Arc<Shared>,
    game: Mutex<GameSide>,
    render: Mutex<RenderSide>,
    exports: Arc<ExportService>,
    biomes: BiomeColors,
    // block name and packed colour, in name order, for the probe
    block_colors: Vec<(String, u32)>,
    capture: Mutex<Option<CaptureWriter>>,
    sink: Option<CaptureSink>,
    telemetry: Option<Mutex<TelemetryRecorder>>,
    telemetry_path: PathBuf,
    // false while the manager has the mod disabled: no scan, no drawing
    active: AtomicBool,
    // the mod was disabled and the cursor clip is not yet given back
    release_cursor: AtomicBool,
}

impl Runtime {
    const MIN_KNOWN_EMPTY: usize = 16_384;
    const MIB: usize = 1024 * 1024;
    pub const TELEMETRY_FILE: &'static str = "telemetry.csv";
    pub const CAPTURE_DIR: &'static str = "captures";

    // `grant` is the texture budget the mod manager gave this mod; it caps the GPU tiles
    pub fn new(config: Config, paths: RuntimePaths, grant: TextureGrant) -> Result<Self, RuntimeError> {
        let biomes = BiomeColors::from_overrides(&config.terrain.biome_color_overrides);
        let settings = BakeSettings::new(&config.terrain);
        let memory = &config.memory;
        let chunk_capacity =
            ChunkStore::capacity_for_budget(memory.chunk_surface_budget_mb as usize * Self::MIB, settings.scale);
        let tile_capacity = TileStore::capacity_for_budget(memory.tile_budget_mb as usize * Self::MIB);
        let shared = Arc::new(Shared::new(
            ChunkStore::new(chunk_capacity),
            TileStore::new(tile_capacity, memory.tile_overrun_percent.max(0) as usize),
            KnownEmptySet::new((tile_capacity * 4).max(Self::MIN_KNOWN_EMPTY)),
            settings,
            ViewState {
                show_coordinates: config.mini_map.show_coordinates || config.world_map.show_coordinates,
                show_chunk_grid: config.world_map.show_chunk_grid,
                show_slime_chunks: config.world_map.show_slime_chunks,
                ..ViewState::new(config.mini_map.radius_chunks)
            },
            WaypointStore::new(config.waypoints.max_per_world.max(1) as usize, Self::seed()),
            MapUi::new(config.mini_map.widget_scale),
        ));
        *Shared::lock(&shared.hover_biomes) =
            HoverBiomeCache::new(config.world_map.hover_biome_regions.max(1) as usize);
        let block_colors = config
            .terrain
            .block_colors
            .iter()
            .map(|(name, [r, g, b, a])| (name.clone(), Rgba::new(*r, *g, *b, *a).packed()))
            .collect();
        let capture = Self::start_capture(&config, &paths);
        let telemetry = config.debug.telemetry.then(|| {
            let interval = Duration::from_millis(config.debug.telemetry_interval_ms.max(0) as u64);
            Mutex::new(TelemetryRecorder::new(interval, Instant::now()))
        });
        let telemetry_path = paths.data_dir.join(Self::TELEMETRY_FILE);
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
        let export_dir = ExportService::folder(&config.export.folder, home.as_deref(), &paths.data_dir);
        let exports = Arc::new(ExportService::start(&export_dir, Arc::clone(&shared))?);
        let render = RenderSide::new(Arc::clone(&shared), &config, Arc::clone(&exports), grant);
        let game = GameSide::new(Arc::clone(&shared), config, paths, Arc::clone(&exports))?;
        if telemetry.is_some() {
            game.append_line(&telemetry_path, TelemetryRow::HEADER.to_string(), true);
        }
        Ok(Self {
            shared,
            game: Mutex::new(game),
            render: Mutex::new(render),
            exports,
            biomes,
            block_colors,
            sink: capture.as_ref().map(CaptureWriter::sink),
            capture: Mutex::new(capture),
            telemetry,
            telemetry_path,
            active: AtomicBool::new(true),
            release_cursor: AtomicBool::new(false),
        })
    }

    pub fn enter_world(&self, identity: &WorldIdentity, dim: i32) -> Result<(), RuntimeError> {
        self.record(|| CaptureEvent::Enter { dim });
        let config = {
            let mut game = Shared::lock(&self.game);
            game.enter_world(identity, dim)?;
            game.config().clone()
        };
        Shared::lock(&self.render).apply_world(&config);
        Ok(())
    }

    // a config the manager changed: live settings now, world settings at the next world, the rest at restart
    pub fn apply_settings(&self, config: Config) {
        let live = Shared::lock(&self.game).apply_settings(config);
        Shared::lock(&self.render).apply_live(&live);
        let mut view = Shared::lock(&self.shared.view);
        view.minimap_radius = live.mini_map.radius_chunks;
        view.show_coordinates = live.mini_map.show_coordinates || live.world_map.show_coordinates;
        view.show_chunk_grid = live.world_map.show_chunk_grid;
        view.show_slime_chunks = live.world_map.show_slime_chunks;
    }

    // the in-play changes to save as manager settings since the last call
    pub fn take_setting_writes(&self, out: &mut Vec<SettingWrite>) {
        Shared::lock(&self.game).take_setting_writes(out);
    }

    pub fn leave_world(&self) {
        self.record(|| CaptureEvent::Leave);
        Shared::lock(&self.game).leave_world();
    }

    pub fn tick(&self, input: &TickInput, plan: &mut ScanPlan) {
        self.record(|| CaptureEvent::Tick(*input));
        let started = Instant::now();
        // a row is written before this tick runs, so it closes the previous tick with its probing
        if let Some(telemetry) = &self.telemetry
            && Shared::lock(telemetry).due(started)
        {
            let stats = self.stats();
            let row = Shared::lock(telemetry).take_row(started, &stats, self.shared.bake_timing.totals());
            Shared::lock(&self.game).append_line(&self.telemetry_path, row.to_csv(), false);
        }
        if !self.active.load(Ordering::SeqCst) {
            plan.candidates.clear();
            plan.cursor = if self.release_cursor.swap(false, Ordering::SeqCst) {
                CursorAction::Release
            } else {
                CursorAction::None
            };
            return;
        }
        if let Some(telemetry) = &self.telemetry {
            Shared::lock(telemetry).on_memory(input.working_set_bytes, input.private_bytes);
        }
        Shared::lock(&self.game).tick(input, plan);
        if let Some(telemetry) = &self.telemetry {
            Shared::lock(telemetry).on_tick(started, Instant::now());
        }
    }

    pub fn submit_chunk(&self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe) {
        self.record(|| CaptureEvent::Submit { pos, status, probe: Box::new(*probe) });
        if status == SubmitStatus::Loaded
            && let Some(telemetry) = &self.telemetry
        {
            Shared::lock(telemetry).on_probe(Instant::now());
        }
        Shared::lock(&self.game).submit_chunk(pos, status, probe);
    }

    pub fn block_changed(&self, x: i32, z: i32, dim: i32) -> ProbeRequest {
        self.record(|| CaptureEvent::Block { x, z, dim });
        Shared::lock(&self.game).block_changed(x, z, dim)
    }

    pub fn submit_column(&self, pos: ChunkPos, index: usize, sample: ColumnSample) {
        self.record(|| CaptureEvent::Column { pos, index: index as u32, sample });
        Shared::lock(&self.game).submit_column(pos, index, sample);
    }

    // the game's internal name for a biome ID the plan asked about; empty when it has none
    pub fn biome_name(&self, id: u16, name: &str) {
        self.record(|| CaptureEvent::BiomeName { id, name: name.to_string() });
        Shared::lock(&self.game).biome_name(id, name);
    }

    // the internal biome name of a resident column on the shown layer
    pub fn column_biome(&self, block_x: i32, block_z: i32) -> Option<String> {
        let layer = self.shared.layer.load(std::sync::atomic::Ordering::Relaxed);
        let key = self.shared.chunks.read().ok()?.column_biome(block_x, block_z, layer)?;
        Shared::lock(&self.shared.biome_keys).name(key).map(str::to_string)
    }

    // colour of a column with no blocks: the biome's terrain.biomeColorOverrides entry, else the game's
    pub fn resolve_void_color(&self, biome: &str, fallback: u32) -> u32 {
        self.biomes.lookup(biome).map_or(fallback, |c| c.packed())
    }

    // colours the probe draws in place of the game's map colour, by block name
    pub fn block_colors(&self) -> &[(String, u32)] {
        &self.block_colors
    }

    pub fn view_flags(&self) -> ViewFlags {
        let view = Shared::lock(&self.shared.view);
        ViewFlags { world_map_open: view.world_map_open, show_minimap: view.show_minimap }
    }

    // the manager took the world map's lease away, or refused it
    pub fn close_world_map(&self) {
        InputController::close_world_map(&mut Shared::lock(&self.shared.view));
        Shared::lock(&self.shared.ui).close();
    }

    pub fn set_active(&self, active: bool) {
        self.release_cursor.store(!active, Ordering::SeqCst);
        self.active.store(active, Ordering::SeqCst);
    }

    // one manager layer pass on the render thread; nothing is drawn while the mod is inactive
    pub fn paint_layer(&self, ctx: &egui::Context, input: &LayerInput) -> LayerReply {
        if !self.active.load(Ordering::SeqCst) {
            return LayerReply::EMPTY;
        }
        let started = Instant::now();
        let (reply, (upload_us, upload_bytes)) = {
            let mut render = Shared::lock(&self.render);
            let reply = render.paint_layer(ctx, input);
            (reply, render.upload_stats())
        };
        if let Some(telemetry) = &self.telemetry {
            let paint_us = u32::try_from(started.elapsed().as_micros()).unwrap_or(u32::MAX);
            Shared::lock(telemetry).on_paint(paint_us, upload_us, upload_bytes);
        }
        reply
    }

    // the manager's whole draw for this mod in one frame, as its frame handler timed it
    pub fn record_render(&self, render_us: u32) {
        if let Some(telemetry) = &self.telemetry {
            Shared::lock(telemetry).on_render(render_us);
        }
    }

    // the manager lost every texture; each layer uploads its own again
    pub fn textures_lost(&self) {
        Shared::lock(&self.render).textures_lost();
    }

    // text a menu choice put on the clipboard, once
    pub fn take_clipboard(&self) -> Option<String> {
        Shared::lock(&self.render).take_clipboard()
    }

    // what the last pass of `layer` drew and uploaded, for tests and benches
    pub fn with_last_paint<R>(&self, layer: MapLayer, read: impl FnOnce(&LayerPaint) -> R) -> R {
        read(Shared::lock(&self.render).last_paint(layer))
    }

    pub fn keybind(&self, input: KeybindInput) -> KeyResult {
        self.record(|| CaptureEvent::Keybind(input));
        Shared::lock(&self.game).keybind(input)
    }

    // a copy of the current world's waypoints, every dimension
    pub fn waypoints(&self) -> Vec<Waypoint> {
        Shared::lock(&self.shared.waypoints).waypoints().to_vec()
    }

    // the current world's waypoints file has been read, so changes are being saved
    pub fn waypoints_loaded(&self) -> bool {
        Shared::lock(&self.game).waypoints_loaded()
    }

    pub fn stats(&self) -> RuntimeStats {
        let base = Shared::lock(&self.game).stats();
        let (tiles_resident, tile_capacity, tile_overrun) = {
            let tiles = Shared::lock(&self.shared.tiles);
            (tiles.len(), tiles.capacity(), tiles.overrun())
        };
        let (surfaces_resident, surface_capacity) =
            self.shared.chunks.try_read().map_or((0, 0), |c| (c.len(), c.capacity()));
        let render = Shared::lock(&self.render);
        RuntimeStats {
            frames: render.frames(),
            gpu_textures: render.gpu_textures(),
            gpu_texture_mb: render.gpu_texture_bytes() / Self::MIB,
            gpu_texture_budget_mb: render.gpu_texture_budget_bytes() / Self::MIB,
            surfaces_resident,
            surface_capacity,
            tiles_resident,
            tile_capacity,
            tile_overrun,
            ..base
        }
    }

    pub fn shutdown(&self) {
        Shared::lock(&self.game).shutdown();
        self.exports.shutdown();
        if let Some(writer) = Shared::lock(&self.capture).take() {
            let report = writer.finish();
            curia::info!("capture closed", { written: report.written, dropped: report.dropped });
        }
    }

    // waypoint ids start from the clock, so two sessions do not make the same ids
    fn seed() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64)
    }

    fn start_capture(config: &Config, paths: &RuntimePaths) -> Option<CaptureWriter> {
        if !config.debug.capture_probes {
            return None;
        }
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let path = paths.data_dir.join(Self::CAPTURE_DIR).join(format!("capture-{secs}.lmcap"));
        match CaptureWriter::start(&path, &CaptureHeader::new(&config.terrain)) {
            Ok(writer) => {
                curia::info!("recording a capture", { path: path.display().to_string() });
                Some(writer)
            }
            Err(error) => {
                curia::warn!("capture not started", { error: error.to_string() });
                None
            }
        }
    }

    // builds the event only when a capture is being recorded
    fn record(&self, event: impl FnOnce() -> CaptureEvent) {
        if let Some(sink) = &self.sink {
            sink.record(event());
        }
    }
}
