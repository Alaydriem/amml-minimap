use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use egui::{Context, Rect, pos2};
use minimap_core::config::{Config, SmoothCameraConfig};
use minimap_core::services::camera::SmoothCamera;
use minimap_core::services::cave::{LayerIds, LayerSelector};
use minimap_core::services::draw::{
    CaveBadgePainter, ChunkGridPainter, ExploredMaskCache, GpuResidency, MapRotation, MinimapComposer, MinimapPainter,
    RegionOrder, SlimeOverlayPainter, TextScale, TextureBudget, WaypointPainter, WorldMapLayout, WorldMapPainter,
};
use minimap_core::services::input::InputController;
use minimap_core::services::ui::WaypointFilter;
use minimap_core::structs::draw::{
    DrawList, ExploredChunks, MapLayer, MinimapFrame, ResidencyDecision, TextureGrant, TextureOp, WorldMapFrame,
};
use minimap_core::structs::export::{ExportKind, ViewRegions};
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::tile::Tile;
use minimap_core::structs::ui::{HintKeys, HoverInfo, MapAction, MenuTarget, UiFrame};
use minimap_core::structs::view::{CameraPose, PlayerSnapshot, ViewState};
use minimap_core::structs::waypoint::Waypoint;
use minimap_ui::map::LayerTextures;
use minimap_ui::runtime::LayerPainter;

use crate::services::{ExportService, HoverBiomeReader, WaypointSync};
use crate::structs::{LayerInput, LayerPaint, LayerReply, Shared};

// everything the render thread calls, one manager layer at a time: the corner map on Hud, the world map and
// its panels on Overlay. It try-locks the shared map data and briefly locks the player, the view and the UI
pub struct RenderSide {
    shared: Arc<Shared>,
    camera: SmoothCamera,
    // the settings the camera was made from; a changed config makes a new camera only when they change
    camera_config: SmoothCameraConfig,
    composer: MinimapComposer,
    minimap: MinimapPainter,
    waypoint_painter: WaypointPainter,
    // the render thread's copy of the waypoints, refreshed when the store's revision moves
    waypoints: Vec<Waypoint>,
    waypoint_revision: Option<u64>,
    prev_right: bool,
    // the left button went down on the menu, the dialog or a panel, so it does not pan the map until released
    left_to_ui: bool,
    exports: Arc<ExportService>,
    // gives a cave layer's band its block range for the badge
    layers: LayerSelector,
    residency: GpuResidency,
    // which visible chunks have map data, for the overlays; keeps its capacity
    explored: ExploredChunks,
    explored_masks: ExploredMaskCache,
    input: InputController,
    regions: Vec<(RegionPos, Option<u32>)>,
    // the regions the world map showed last, for Export this view
    last_view: Option<ViewRegions>,
    // the visible regions' positions and their nearest-first order this frame; both keep their capacity
    visible: Vec<RegionPos>,
    order: Vec<usize>,
    frame: u64,
    epoch: u64,
    minimap_uploaded: bool,
    prev_cursor: (f32, f32),
    prev_left: bool,
    font_size: f32,
    widget_scale: f32,
    // font_size times the screen's text scale, for this frame
    text_size: f32,
    show_world_nether: bool,
    rotate_minimap: bool,
    rotate_world_map: bool,
    max_updates: usize,
    grid: ChunkGridPainter,
    slime: SlimeOverlayPainter,
    hover: HoverBiomeReader,
    filter: WaypointFilter,
    // indices into `waypoints` that the list shows this frame; keeps its capacity
    list_rows: Vec<usize>,
    hud_painter: LayerPainter,
    overlay_painter: LayerPainter,
    hud_textures: LayerTextures,
    overlay_textures: LayerTextures,
    hud: LayerPaint,
    overlay: LayerPaint,
    // the frame time of the last pass; both layers of one frame carry the same time
    last_time: Option<f64>,
    // the camera pose of this frame
    pose: CameraPose,
    // text for the clipboard from a menu choice, until the game thread takes it
    clipboard: Option<String>,
    // the block under the cursor on the world map, when the cursor is on the map
    hovered: Option<(i32, i32)>,
    player_block: (i32, i32, i32),
    // the chord that opens and closes the world map, for the hint bar
    toggle_label: String,
    // the panels' requests from this pass, applied after it; keeps its capacity
    actions: Vec<MapAction>,
    // the last pass's texture upload time and bytes, for telemetry
    upload_us: u32,
    upload_bytes: u64,
}

impl RenderSide {
    const IDLE_CHECK_FRAMES: u64 = 300;
    // about once a second, regions that left memory give back their GPU tiles
    const UNLOADED_CHECK_FRAMES: u64 = 60;
    // the cave badge's top, below the top of the minimap disc or the screen
    const BADGE_INSET: f32 = 8.0;
    // the height given to a waypoint added on a column that has not been explored
    pub const DEFAULT_Y: i32 = 64;
    // the longest step the camera and the message line take in one frame
    const MAX_DT: f32 = 0.25;

    pub fn new(shared: Arc<Shared>, config: &Config, exports: Arc<ExportService>, grant: TextureGrant) -> Self {
        let world = &config.world_map;
        let tiles = TextureBudget::tiles(grant, config.memory.gpu_texture_budget_mb);
        Self {
            shared,
            camera: SmoothCamera::new(&config.smooth_camera),
            camera_config: config.smooth_camera.clone(),
            composer: MinimapComposer::new(),
            minimap: MinimapPainter::new(&config.mini_map),
            waypoint_painter: WaypointPainter::new(config),
            waypoints: Vec::new(),
            waypoint_revision: None,
            prev_right: false,
            left_to_ui: false,
            exports,
            layers: LayerSelector::new(config),
            explored: ExploredChunks::new(),
            explored_masks: ExploredMaskCache::new(tiles),
            residency: GpuResidency::new(
                tiles,
                world.texture_idle_frames.max(1) as u64,
                world.max_new_textures_per_frame.max(1) as usize,
            ),
            input: InputController::new(config),
            regions: Vec::new(),
            last_view: None,
            visible: Vec::new(),
            order: Vec::new(),
            frame: 0,
            epoch: 0,
            minimap_uploaded: false,
            prev_cursor: (0.0, 0.0),
            prev_left: false,
            font_size: config.font_size,
            widget_scale: config.mini_map.widget_scale,
            text_size: config.font_size,
            show_world_nether: world.show_nether_coordinates,
            rotate_minimap: config.mini_map.rotate_with_player,
            rotate_world_map: world.rotate_with_player,
            max_updates: world.max_new_textures_per_frame.max(1) as usize * 2,
            grid: ChunkGridPainter::new(&config.mini_map),
            slime: SlimeOverlayPainter::new(world.slime_cache_regions.max(1) as usize),
            hover: HoverBiomeReader::new(),
            filter: WaypointFilter::new(),
            list_rows: Vec::new(),
            hud_painter: LayerPainter::new(),
            overlay_painter: LayerPainter::new(),
            hud_textures: LayerTextures::new(1),
            overlay_textures: LayerTextures::new(tiles),
            hud: LayerPaint::new(),
            overlay: LayerPaint::new(),
            last_time: None,
            pose: CameraPose::default(),
            clipboard: None,
            hovered: None,
            player_block: (0, 0, 0),
            toggle_label: config.keys.toggle_world_map.clone(),
            actions: Vec::new(),
            upload_us: 0,
            upload_bytes: 0,
        }
    }

    // the settings the manager may change while the mod runs
    pub fn apply_live(&mut self, config: &Config) {
        if self.camera_config != config.smooth_camera {
            self.camera = SmoothCamera::new(&config.smooth_camera);
            self.camera_config = config.smooth_camera.clone();
        }
        let world = &config.world_map;
        self.minimap = MinimapPainter::new(&config.mini_map);
        self.waypoint_painter = WaypointPainter::new(config);
        self.input = InputController::new(config);
        self.font_size = config.font_size;
        self.widget_scale = config.mini_map.widget_scale;
        self.show_world_nether = world.show_nether_coordinates;
        self.rotate_minimap = config.mini_map.rotate_with_player;
        self.rotate_world_map = world.rotate_with_player;
        self.grid = ChunkGridPainter::new(&config.mini_map);
        self.toggle_label.clone_from(&config.keys.toggle_world_map);
    }

    // the settings a world takes when it starts
    pub fn apply_world(&mut self, config: &Config) {
        self.layers = LayerSelector::new(config);
    }

    pub fn gpu_textures(&self) -> usize {
        self.residency.len() + usize::from(self.minimap_uploaded)
    }

    pub fn gpu_texture_bytes(&self) -> usize {
        self.residency.resident_bytes()
    }

    pub fn gpu_texture_budget_bytes(&self) -> usize {
        self.residency.budget_bytes()
    }

    pub fn frames(&self) -> u64 {
        self.frame
    }

    // the last pass of `layer`: what it drew and what it uploaded
    pub fn last_paint(&self, layer: MapLayer) -> &LayerPaint {
        match layer {
            MapLayer::Hud => &self.hud,
            MapLayer::Overlay => &self.overlay,
        }
    }

    // the last pass's texture upload time in microseconds and bytes
    pub fn upload_stats(&self) -> (u32, u64) {
        (self.upload_us, self.upload_bytes)
    }

    pub fn take_clipboard(&mut self) -> Option<String> {
        self.clipboard.take()
    }

    // the manager lost every texture: both layers upload theirs again
    pub fn textures_lost(&mut self) {
        self.drop_textures();
    }

    pub fn paint_layer(&mut self, ctx: &Context, input: &LayerInput) -> LayerReply {
        if self.last_time != Some(input.time_secs) {
            self.begin_frame(input);
        }
        let display = (input.width as f32, input.height as f32);
        let player = *Shared::lock(&self.shared.player);
        let view = *Shared::lock(&self.shared.view);
        let reply = match input.layer {
            MapLayer::Hud => {
                self.hud.clear();
                if !player.present || view.world_map_open {
                    return LayerReply::EMPTY;
                }
                if self.hud_textures.ensure_context(ctx) {
                    self.minimap_uploaded = false;
                    self.composer.invalidate();
                }
                self.paint_hud(ctx, display, &player, &view)
            }
            MapLayer::Overlay => {
                self.overlay.clear();
                if !player.present || !view.world_map_open {
                    self.prev_left = false;
                    return LayerReply::EMPTY;
                }
                if self.overlay_textures.ensure_context(ctx) {
                    self.residency.release_all(&mut self.overlay.stage.ops);
                    self.overlay.stage.clear();
                }
                self.paint_overlay(ctx, display, &player)
            }
        };
        self.take_clipboard_from_ui();
        reply
    }

    // once per frame, on the first layer pass: time, camera, message line, exports, world change
    fn begin_frame(&mut self, input: &LayerInput) {
        let dt = self.last_time.map_or(0.0, |last| (input.time_secs - last) as f32).clamp(0.0, Self::MAX_DT);
        self.last_time = Some(input.time_secs);
        self.frame += 1;
        self.hovered = None;
        self.drop_textures_on_world_change();
        let player = *Shared::lock(&self.shared.player);
        if player.present {
            self.pose = self.camera.update(&player, dt);
        }
        let text_scale = TextScale::for_display(input.height as f32);
        self.text_size = self.font_size * text_scale;
        let map_open = Shared::lock(&self.shared.view).world_map_open;
        let mut ui = Shared::lock(&self.shared.ui);
        ui.set_scale(self.widget_scale * text_scale);
        ui.tick(dt);
        while let Some(message) = self.exports.take_result() {
            if message.starts_with(ExportService::EXPORTED) {
                ui.show_done(message);
            } else {
                ui.show_message(message);
            }
        }
        match ui.take_export() {
            Some(ExportKind::View) if self.exports.is_busy() => {
                ui.show_message("An export is already running".to_string())
            }
            Some(ExportKind::View) => *Shared::lock(&self.shared.view_export) = self.last_view,
            Some(ExportKind::Area) => self.shared.area_export.store(true, Ordering::SeqCst),
            None => {}
        }
        if !map_open && ui.is_open() {
            ui.close();
        }
        drop(ui);
        self.refresh_waypoints();
    }

    fn paint_hud(
        &mut self,
        ctx: &Context,
        display: (f32, f32),
        player: &PlayerSnapshot,
        view: &ViewState,
    ) -> LayerReply {
        let mut rect = Rect::NOTHING;
        if view.show_minimap {
            self.minimap(display, player, view);
            let ((x, y), radius) = self.minimap.disc(display.0);
            let ppp = ctx.pixels_per_point().max(f32::EPSILON);
            rect = Rect::from_min_max(
                pos2((x - radius) / ppp, (y - radius) / ppp),
                pos2((x + radius) / ppp, (y + radius) / ppp),
            );
        }
        self.upload(ctx, MapLayer::Hud);
        let ui = Shared::lock(&self.shared.ui);
        let frame = UiFrame {
            panels: false,
            ui: &ui,
            view,
            waypoints: &self.waypoints,
            rows: &self.list_rows,
            in_dimension: 0,
            player_block: player.block(),
            dim: player.dim,
            hover: None,
            keys: HintKeys { toggle: &self.toggle_label },
        };
        self.hud_painter.paint(ctx, MapLayer::Hud, &self.hud.draw, &self.hud_textures, Some(&frame));
        LayerReply { rect, arrow_cursor: false }
    }

    fn paint_overlay(&mut self, ctx: &Context, display: (f32, f32), player: &PlayerSnapshot) -> LayerReply {
        let ppp = ctx.pixels_per_point().max(f32::EPSILON);
        let (cursor, left_down, right_down) = ctx.input(|input| {
            let at = input.pointer.latest_pos().map(|p| (p.x * ppp, p.y * ppp));
            (at, input.pointer.primary_down(), input.pointer.secondary_down())
        });
        let pointer_here = cursor.is_some();
        let cursor = cursor.unwrap_or(self.prev_cursor);
        let presses = (left_down && !self.prev_left, right_down && !self.prev_right);
        if !left_down {
            self.left_to_ui = false;
        }
        let over_panel = LayerPainter::covers(ctx, cursor);
        let view = {
            let mut view = Shared::lock(&self.shared.view);
            if pointer_here && left_down && self.prev_left && !self.left_to_ui {
                let (dx, dy) = (cursor.0 - self.prev_cursor.0, cursor.1 - self.prev_cursor.1);
                self.input.on_drag(&mut view, dx, dy, Self::angle(self.rotate_world_map, self.pose));
            }
            if pointer_here && !over_panel {
                self.input.on_wheel(&mut view, LayerPainter::wheel_lines(ctx));
            }
            *view
        };
        self.prev_cursor = cursor;
        self.prev_left = left_down;
        self.prev_right = right_down;
        let layer = self.world_map(display, player, &view, (pointer_here, cursor, over_panel), presses);
        self.upload(ctx, MapLayer::Overlay);
        self.overlay_pass(ctx, player, &view, layer);
        LayerReply {
            rect: Rect::from_min_max(pos2(0.0, 0.0), pos2(display.0 / ppp, display.1 / ppp)),
            arrow_cursor: true,
        }
    }

    // the world map's panels over its draw list, then the panels' requests are applied
    fn overlay_pass(&mut self, ctx: &Context, player: &PlayerSnapshot, view: &ViewState, layer: i32) {
        let dim = player.dim;
        self.filter.set_query(self.overlay_painter.search());
        self.filter.collect(&self.waypoints, dim, &mut self.list_rows);
        {
            let ui = Shared::lock(&self.shared.ui);
            let biome = self.hovered.and_then(|block| self.hover.read(&self.shared, block, layer));
            let hover = self.hovered.map(|block| HoverInfo {
                block,
                chunk: (block.0.div_euclid(16), block.1.div_euclid(16)),
                biome,
            });
            let frame = UiFrame {
                panels: true,
                ui: &ui,
                view,
                waypoints: &self.waypoints,
                rows: &self.list_rows,
                in_dimension: WaypointFilter::count_in(&self.waypoints, dim),
                player_block: self.player_block,
                dim,
                hover,
                keys: HintKeys { toggle: &self.toggle_label },
            };
            let actions = self.overlay_painter.paint(
                ctx,
                MapLayer::Overlay,
                &self.overlay.draw,
                &self.overlay_textures,
                Some(&frame),
            );
            self.actions.clear();
            self.actions.extend_from_slice(actions);
        }
        self.apply_actions(dim);
        self.shared.typing.store(self.overlay_painter.typing(), Ordering::Relaxed);
    }

    // hands this pass's uploads to the layer's egui context and times it
    fn upload(&mut self, ctx: &Context, layer: MapLayer) {
        let started = Instant::now();
        let (paint, textures) = match layer {
            MapLayer::Hud => (&self.hud, &mut self.hud_textures),
            MapLayer::Overlay => (&self.overlay, &mut self.overlay_textures),
        };
        textures.apply(ctx, &paint.stage.ops, paint.stage.live_buffers());
        self.upload_bytes = paint.stage.bytes;
        self.upload_us = u32::try_from(started.elapsed().as_micros()).unwrap_or(u32::MAX);
    }

    fn apply_actions(&mut self, dim: i32) {
        if self.actions.is_empty() {
            return;
        }
        let closes = self.actions.contains(&MapAction::CloseWorldMap);
        {
            let mut ui = Shared::lock(&self.shared.ui);
            let mut view = Shared::lock(&self.shared.view);
            let mut store = Shared::lock(&self.shared.waypoints);
            let name = &self.overlay_painter.drafts().name;
            for action in &self.actions {
                ui.apply(*action, name, dim, &mut view, &mut store, WaypointSync::now());
            }
        }
        if closes {
            self.overlay_painter.reset_drafts();
        }
    }

    fn take_clipboard_from_ui(&mut self) {
        if let Some(text) = Shared::lock(&self.shared.ui).take_clipboard() {
            self.clipboard = Some(text);
        }
    }

    // the column under a world map screen point; y is its stored height, or DEFAULT_Y when not explored
    // `layer` is the map layer whose stored height is read; the target keeps the real dimension
    fn ground(shared: &Shared, frame: &WorldMapFrame<'_>, layer: i32, at: (f32, f32)) -> MenuTarget {
        let (x, z) = WorldMapLayout::block_at(at, frame.centre, frame.zoom, frame.display, frame.angle);
        let height = shared.chunks.try_read().ok().and_then(|chunks| chunks.column_height(x, z, layer));
        MenuTarget::Ground { dim: frame.dim, x, y: height.map_or(Self::DEFAULT_Y, i32::from), z }
    }

    // the badge of the shown cave band, if one is shown
    fn cave_badge(&self, top_centre: (f32, f32), draw: &mut DrawList) {
        let layer = self.shared.layer.load(Ordering::Relaxed);
        Self::badge_for(&self.layers, layer, top_centre, self.text_size, draw);
    }

    fn badge_for(layers: &LayerSelector, layer: i32, top_centre: (f32, f32), size: f32, draw: &mut DrawList) {
        if let Some(band) = LayerIds::band(layer) {
            CaveBadgePainter::paint(draw, top_centre, size, (layers.band_bottom(band), layers.band_top(band)));
        }
    }

    // the map turn for a map with `rotate` set: the player's facing at the top
    fn angle(rotate: bool, pose: CameraPose) -> f32 {
        if rotate { MapRotation::player_up(pose.render_yaw_rad()) } else { 0.0 }
    }

    // copies the list only after a change; a busy lock keeps last frame's copy
    fn refresh_waypoints(&mut self) {
        let Ok(store) = self.shared.waypoints.try_lock() else { return };
        if self.waypoint_revision != Some(store.revision()) {
            self.waypoint_revision = Some(store.revision());
            self.waypoints.clear();
            self.waypoints.extend_from_slice(store.waypoints());
        }
    }

    fn drop_textures_on_world_change(&mut self) {
        let epoch = self.shared.world_epoch.load(Ordering::SeqCst);
        if epoch == self.epoch {
            return;
        }
        self.epoch = epoch;
        self.drop_textures();
        self.explored_masks.clear();
        self.camera.reset();
        self.hover.reset();
        self.overlay_painter.reset_drafts();
        self.shared.typing.store(false, Ordering::Relaxed);
    }

    // every texture of both layers goes; each is uploaded again when next drawn
    fn drop_textures(&mut self) {
        let mut released = Vec::new();
        self.residency.release_all(&mut released);
        self.hud_textures.clear();
        self.overlay_textures.clear();
        self.minimap_uploaded = false;
        self.composer.invalidate();
    }

    fn minimap(&mut self, display: (f32, f32), player: &PlayerSnapshot, view: &ViewState) {
        let pose = self.pose;
        let centre = (pose.x.floor() as i32, pose.z.floor() as i32);
        if let Ok(tiles) = self.shared.tiles.try_lock() {
            let chunks = self.shared.chunks.try_read().ok();
            let layer = self.shared.layer.load(Ordering::Relaxed);
            if self.composer.update(centre, layer, view.minimap_radius, &tiles, chunks.as_deref()) {
                let side = self.composer.side() as u32;
                self.hud.stage.upload(GpuResidency::MINIMAP_TEXTURE, side, side, self.composer.image());
                self.minimap_uploaded = true;
            }
        }
        let frame = MinimapFrame {
            display_width: display.0,
            camera: pose,
            player_block: player.block(),
            radius_chunks: view.minimap_radius,
            font_size: self.text_size,
            texture: self.minimap_uploaded.then_some(GpuResidency::MINIMAP_TEXTURE),
            image_origin: self.composer.origin(),
            image_side: self.composer.side(),
            dim: player.dim,
            show_coordinates: view.show_coordinates,
            angle: Self::angle(self.rotate_minimap, pose),
        };
        let mut draw = std::mem::take(&mut self.hud.draw);
        self.minimap.paint(&frame, &mut draw);
        let disc = self.minimap.disc(display.0);
        self.cave_badge((disc.0.0, disc.0.1 - disc.1 + Self::BADGE_INSET), &mut draw);
        let shown = Shared::lock(&self.shared.ui).waypoints_shown();
        let waypoints: &[Waypoint] = if shown { &self.waypoints } else { &[] };
        self.waypoint_painter.paint_minimap(&frame, disc, waypoints, &mut draw);
        self.hud.draw = draw;
    }

    // the world map's draw list and texture work; `pointer` is (on this layer, position, over a panel).
    // Returns the shown layer
    fn world_map(
        &mut self,
        display: (f32, f32),
        player: &PlayerSnapshot,
        view: &ViewState,
        pointer: (bool, (f32, f32), bool),
        presses: (bool, bool),
    ) -> i32 {
        let (pointer_here, cursor, over_panel) = pointer;
        let pose = self.pose;
        let centre = view.centre((pose.x, pose.z));
        let angle = Self::angle(self.rotate_world_map, pose);
        let layer = self.shared.layer.load(Ordering::Relaxed);
        let (rx0, rx1, rz0, rz1) = if angle == 0.0 {
            WorldMapLayout::visible(centre, view.zoom, display)
        } else {
            WorldMapLayout::visible_turned(centre, view.zoom, display)
        };
        self.last_view = Some(ViewRegions { x0: rx0, x1: rx1, z0: rz0, z1: rz1, layer });
        self.residency.begin_frame(self.frame);
        self.explored.begin((rx0, rz0), rx1 - rx0 + 1);
        self.regions.clear();
        self.visible.clear();
        let mut updates = 0;
        {
            let tiles = self.shared.tiles.try_lock().ok();
            let known_empty = self.shared.known_empty.try_lock().ok();
            for rz in rz0..=rz1 {
                for rx in rx0..=rx1 {
                    let region = RegionPos::new(rx, rz, layer);
                    let tile = tiles.as_ref().and_then(|t| t.peek(region));
                    self.explored.push(self.explored_masks.mask(region, tile));
                    self.visible.push(region);
                    self.regions.push((region, None));
                }
            }
            // the nearest regions get textures first, so a full budget leaves out the far edge of the view
            RegionOrder::nearest_first(&self.visible, centre, &mut self.order);
            let stage = &mut self.overlay.stage;
            for &index in &self.order {
                {
                    let region = self.visible[index];
                    let tile = tiles.as_ref().and_then(|t| t.peek(region));
                    let decision = self.residency.decide(region, tile.map(Tile::version), &mut stage.ops);
                    let tex = match (decision, tile) {
                        (ResidencyDecision::Draw(tex), _) => Some(tex),
                        (ResidencyDecision::Upload { tex, created }, Some(tile))
                            if created || updates < self.max_updates =>
                        {
                            stage.upload(tex, 256, 256, tile.pixels());
                            self.residency.uploaded(region, tile.version());
                            updates += 1;
                            Some(tex)
                        }
                        (ResidencyDecision::Upload { tex, .. }, _) => Some(tex),
                        (ResidencyDecision::Placeholder, None) => {
                            if known_empty.as_ref().is_some_and(|k| !k.contains(region)) {
                                let _ = self.shared.tile_requests.0.try_send(region);
                            }
                            None
                        }
                        (ResidencyDecision::Placeholder, Some(_)) => None,
                    };
                    self.regions[index].1 = tex;
                }
            }
            if self.frame.is_multiple_of(Self::UNLOADED_CHECK_FRAMES)
                && let Some(tiles) = tiles.as_ref()
            {
                self.residency.release_unloaded(|region| tiles.peek(region).is_some(), &mut stage.ops);
            }
        }
        if self.frame.is_multiple_of(Self::IDLE_CHECK_FRAMES) {
            let mut released: Vec<TextureOp> = Vec::new();
            self.residency.release_idle(&mut released);
            self.overlay.stage.ops.extend_from_slice(&released);
        }
        let frame = WorldMapFrame {
            display,
            centre,
            zoom: view.zoom,
            camera: pose,
            player_block: player.block(),
            font_size: self.text_size,
            show_coordinates: view.show_coordinates,
            show_nether_coordinates: self.show_world_nether,
            dim: player.dim,
            angle,
            regions: &self.regions,
        };
        let mut draw = std::mem::take(&mut self.overlay.draw);
        WorldMapPainter::paint(&frame, &mut draw);
        let hovered =
            (pointer_here && !over_panel).then(|| WorldMapLayout::block_at(cursor, centre, view.zoom, display, angle));
        if view.show_slime_chunks {
            self.slime.paint(&frame, &self.explored, &mut draw);
        }
        if view.show_chunk_grid {
            let chunk = hovered.map(|(x, z)| (x.div_euclid(16), z.div_euclid(16)));
            self.grid.paint(&frame, chunk, &self.explored, &mut draw);
        }
        let shown = Shared::lock(&self.shared.ui).waypoints_shown();
        let waypoints: &[Waypoint] = if shown { &self.waypoints } else { &[] };
        self.waypoint_painter.paint_world_map(&frame, waypoints, &mut draw);
        Self::badge_for(&self.layers, layer, (display.0 * 0.5, Self::BADGE_INSET), self.text_size, &mut draw);
        let (left_pressed, right_pressed) = presses;
        if right_pressed && pointer_here && !over_panel {
            let target = match self.waypoint_painter.hit(&frame, waypoints, cursor) {
                Some(id) => MenuTarget::Marker(id),
                None => Self::ground(&self.shared, &frame, layer, cursor),
            };
            Shared::lock(&self.shared.ui).open_menu(cursor, target);
        }
        self.overlay.draw = draw;
        self.hovered = hovered;
        self.player_block = player.block();
        if left_pressed {
            // an open menu or dialog takes every press, as do the panels
            self.left_to_ui = over_panel || Shared::lock(&self.shared.ui).is_open();
        }
        layer
    }
}
