use std::path::Path;

use amml_api::{BlockSource, Client, ColumnSample as GameSample, Ctx, Window};
use bytemuck::Zeroable;
use minimap_core::structs::chunk::ChunkProbe;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_os::services::MemoryInfo;
use minimap_runtime::structs::{ProbeRequest, ScanPlan, SubmitStatus, TickInput};
use minimap_store::structs::WorldIdentity;

use crate::services::{BlockChangeQueue, ColumnProbe, CursorClip, MinimapLog, WorldIdentityReader};
use crate::traits::MapSink;
use crate::types::{ColorSource, QueuedColumn};

// follows the player in and out of worlds and dimensions, ticks the minimap, probes its scan plan and reads the
// columns block changes asked for. Every call is for the game thread
pub struct WorldTracker<S: MapSink> {
    sink: S,
    probe: ColumnProbe,
    identity: WorldIdentityReader,
    window: Window,
    cursor: CursorClip,
    memory: MemoryInfo,
    log: MinimapLog,
    changes: BlockChangeQueue,
    plan: ScanPlan,
    staging: Box<ChunkProbe>,
    in_world: bool,
    dimension: i32,
    // the loaded chunks read in this world
    loaded: u64,
}

impl<S: MapSink> WorldTracker<S> {
    // `data_dir`: where the caches of servers and unnamed worlds go
    pub fn new(ctx: &Ctx, sink: S, data_dir: &Path) -> Self {
        Self {
            sink,
            probe: ColumnProbe::new(ctx),
            identity: WorldIdentityReader::new(data_dir),
            window: ctx.window(),
            cursor: CursorClip::new(ctx.window(), ctx.log()),
            memory: MemoryInfo::new(),
            log: MinimapLog::new(ctx.log()),
            changes: BlockChangeQueue::new(),
            plan: ScanPlan::default(),
            staging: Box::new(ChunkProbe::zeroed()),
            in_world: false,
            dimension: -1,
            loaded: 0,
        }
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    pub fn sink_mut(&mut self) -> &mut S {
        &mut self.sink
    }

    pub fn in_world(&self) -> bool {
        self.in_world
    }

    // gives the sink back, for the session's shutdown
    pub fn into_sink(self) -> S {
        self.sink
    }

    // once per client tick, after the game's own update
    pub fn tick(&mut self, client: &Client<'_>) {
        let mouse = client.mouse_captured().unwrap_or(false);
        let window = self.window.is_foreground();
        // without the window read the window counts as in front
        let foreground = window.unwrap_or(true);
        let player = client.local_player().filter(|_| client.world_active() == Some(true));
        let Some(player) = player else {
            self.leave();
            self.tick_without_player(mouse, window);
            return;
        };
        let eye = player.eye_position();
        let rotation = player.rotation();
        let dimension = player.dimension().and_then(|dimension| dimension.id());
        let (Some(eye), Some(rotation), Some(dimension)) = (eye, rotation, dimension) else {
            self.tick_without_player(mouse, window);
            return;
        };
        if !self.in_world {
            self.enter(client, dimension);
        } else if dimension != self.dimension {
            self.dimension = dimension;
            self.log.dimension(dimension);
        }
        let (working_set_bytes, private_bytes) = self.memory.sample();
        let input = TickInput {
            player: PlayerSnapshot { present: true, x: eye.x, y: eye.y, z: eye.z, yaw: rotation.yaw, dim: dimension },
            foreground,
            cursor_hidden: mouse,
            dead: player.is_alive() == Some(false),
            mouse_grabbed: mouse,
            working_set_bytes,
            private_bytes,
        };
        let mut plan = std::mem::take(&mut self.plan);
        self.sink.tick(&input, &mut plan);
        self.probe.configure(plan.camera_height, plan.transparent_water);
        if let Some(source) = player.block_source() {
            self.probe_plan(&source, &plan);
            self.read_changes(&source, dimension);
        }
        self.answer_biomes(&plan.biome_requests);
        self.cursor.apply(plan.cursor, mouse, window);
        self.plan = plan;
    }

    // a block of the player's dimension changed type: its column is read at the next tick
    pub fn block_changed(&mut self, column: QueuedColumn) {
        if self.in_world {
            self.changes.push(column);
        }
    }

    // the player left the world: the minimap leaves it now and gives the cursor back
    pub fn leave(&mut self) {
        if !self.in_world {
            return;
        }
        self.sink.leave_world();
        self.cursor.drop_clip("left");
        self.changes.clear();
        self.probe.biomes().clear();
        self.log.world_left();
        self.in_world = false;
        self.dimension = -1;
    }

    // disable: gives the cursor back and leaves the world
    pub fn reset(&mut self) {
        self.cursor.drop_clip("disabled");
        self.leave();
    }

    fn enter(&mut self, client: &Client<'_>, dimension: i32) {
        let identity = self.identity.read(client.world_info().as_ref());
        self.changes.clear();
        self.probe.biomes().clear();
        self.loaded = 0;
        self.log.world_entered(dimension);
        let kind = if matches!(identity, WorldIdentity::Remote { .. }) { "remote" } else { "local" };
        self.log.world_named(kind, self.identity.from_world_folder(&identity));
        if let Err(error) = self.sink.enter_world(&identity, dimension) {
            self.log.logger().error("minimap world not entered", &[("error", &error.to_string())]);
        }
        self.in_world = true;
        self.dimension = dimension;
    }

    // out of a world the minimap still ticks with no player: it drains the manager's events and gives its leases
    // back
    fn tick_without_player(&mut self, mouse: bool, window: Option<bool>) {
        let input = TickInput {
            foreground: window.unwrap_or(true),
            cursor_hidden: mouse,
            mouse_grabbed: mouse,
            ..TickInput::default()
        };
        let mut plan = std::mem::take(&mut self.plan);
        self.sink.tick(&input, &mut plan);
        self.cursor.apply(plan.cursor, mouse, window);
        self.plan = plan;
    }

    // probes the candidates in order and stops reading after `limit` loaded chunks
    fn probe_plan(&mut self, source: &BlockSource<'_>, plan: &ScanPlan) {
        let mut loaded = 0;
        for &pos in &plan.candidates {
            if loaded >= plan.limit {
                self.sink.submit_chunk(pos, SubmitStatus::NotProbed, &self.staging);
                continue;
            }
            *self.staging = ChunkProbe::zeroed();
            let status = self.probe.probe_chunk(source, pos, &self.sink, &mut self.staging);
            if status == SubmitStatus::Loaded {
                loaded += 1;
            }
            self.sink.submit_chunk(pos, status, &self.staging);
        }
        self.loaded += loaded as u64;
        self.log.chunks_read(self.loaded);
        for source in [ColorSource::Vanilla, ColorSource::Table, ColorSource::Fallback] {
            if self.probe.met(source) {
                self.log.colours(source);
            }
        }
    }

    // the columns block changes asked for, of the player's dimension
    fn read_changes(&mut self, source: &BlockSource<'_>, dimension: i32) {
        while let Some(column) = self.changes.pop() {
            if column.dimension != dimension {
                continue;
            }
            let ProbeRequest::Column { pos, index } = self.sink.block_changed(column.x, column.z, dimension) else {
                continue;
            };
            if let Some(sample) = self.probe.probe_column(source, pos, index, &self.sink) {
                self.sink.submit_column(pos, index, sample);
                self.log.block_change_column(column.x, column.z);
            }
        }
    }

    // names the biome ids the plan asks for; a column with no biome id gets an empty name, so it does not wait
    fn answer_biomes(&mut self, requests: &[u16]) {
        for &id in requests {
            if id == GameSample::NO_BIOME {
                self.sink.biome_name(id, "");
                continue;
            }
            if let Some(name) = self.probe.biomes().get(id) {
                self.sink.biome_name(id, name);
                self.log.biome_named(id, name);
            }
        }
    }
}
