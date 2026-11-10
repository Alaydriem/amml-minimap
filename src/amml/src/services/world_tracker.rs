use std::ffi::c_void;
use std::path::Path;
use std::ptr::NonNull;
use std::sync::Arc;

use amml_game::{Actor, BlockSource, ClientInstance, GameEntries};
use amml_msvc::MsvcHeap;
use bytemuck::Zeroable;
use minimap_core::structs::chunk::ChunkProbe;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_runtime::structs::{ProbeRequest, ScanPlan, SubmitStatus, TickInput};

use crate::game::{Level, ListenerSource};
use crate::services::{
    BlockListener, ColumnProbe, CursorClip, GameWindow, ListenerHub, MemoryInfo, WorldIdentityReader, WorldStorage,
};
use crate::traits::MapSink;
use crate::types::{BlockChange, ListenerSlots, MinimapFeature, ProbeSettings};

// follows the player in and out of worlds and dimensions, ticks the minimap and probes its scan plan. Every call
// is for the game thread
pub struct WorldTracker<S: MapSink> {
    entries: Arc<GameEntries>,
    sink: S,
    probe: ColumnProbe,
    identity: WorldIdentityReader,
    storage: &'static WorldStorage,
    hub: &'static ListenerHub,
    heap: Option<MsvcHeap>,
    // the listener's base table and slots; None while the block changes feature is off
    listener_slots: Option<(u64, ListenerSlots)>,
    // made at the first registration, on the game heap
    listener: Option<BlockListener>,
    window: GameWindow,
    cursor: CursorClip,
    memory: MemoryInfo,
    plan: ScanPlan,
    staging: Box<ChunkProbe>,
    in_world: bool,
    dimension: i32,
}

impl<S: MapSink> WorldTracker<S> {
    pub fn new(
        entries: GameEntries,
        sink: S,
        hub: &'static ListenerHub,
        heap: Option<MsvcHeap>,
        data_dir: &Path,
        storage: &'static WorldStorage,
    ) -> Self {
        let vanilla = MinimapFeature::is_on(&entries, MinimapFeature::VANILLA_COLORS);
        let listener_slots = if MinimapFeature::is_on(&entries, MinimapFeature::BLOCK_CHANGES) {
            BlockListener::slots(&entries).filter(|(_, slots)| BlockListener::check(slots).is_ok())
        } else {
            None
        };
        Self {
            entries: Arc::new(entries),
            sink,
            probe: ColumnProbe::new(vanilla),
            identity: WorldIdentityReader::new(data_dir, heap),
            storage,
            hub,
            heap,
            listener_slots,
            listener: None,
            window: GameWindow::new(),
            cursor: CursorClip::new(),
            memory: MemoryInfo::new(),
            plan: ScanPlan::default(),
            staging: Box::new(ChunkProbe::zeroed()),
            in_world: false,
            dimension: -1,
        }
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    pub fn in_world(&self) -> bool {
        self.in_world
    }

    pub fn listener(&self) -> Option<&BlockListener> {
        self.listener.as_ref()
    }

    // gives the sink back, for the session's shutdown
    pub fn into_sink(self) -> S {
        self.sink
    }

    // once per client update, after the game's own: world entry and exit, the minimap tick, the scan plan's probes
    #[allow(unsafe_code)]
    pub fn tick(&mut self, client: NonNull<c_void>) {
        let entries = Arc::clone(&self.entries);
        // SAFETY: the client update detour passes the live ClientInstance the game is updating
        let Some(client) = (unsafe { ClientInstance::from_raw(client.as_ptr(), &entries) }) else { return };
        let player = client.local_player().filter(|_| client.is_world_active());
        let Some(player) = player else {
            self.leave();
            self.tick_without_player(&client);
            return;
        };
        let (Some(pose), Some(dimension)) = (player.pose(), player.dimension().and_then(|d| d.id())) else {
            self.tick_without_player(&client);
            return;
        };
        if !self.in_world {
            self.enter(&entries, &client, &player, dimension);
        } else if dimension != self.dimension || self.listener_lost() {
            self.attach_listener(&entries, &player);
            self.dimension = dimension;
        }
        let (working_set_bytes, private_bytes) = self.memory.sample();
        let [x, y, z] = pose.pos;
        let input = TickInput {
            player: PlayerSnapshot { present: true, x, y, z, yaw: pose.yaw, dim: dimension },
            foreground: self.window.foreground(),
            cursor_hidden: self.cursor.cursor_hidden(),
            dead: player.is_alive() == Some(false),
            mouse_grabbed: client.mouse_grabbed(),
            working_set_bytes,
            private_bytes,
        };
        let mut plan = std::mem::take(&mut self.plan);
        self.sink.tick(&input, &mut plan);
        self.probe.configure(ProbeSettings::new(plan.camera_height, plan.transparent_water));
        // SAFETY: the actor's level is live while the actor is
        if let Some(level) = player.level().and_then(|level| unsafe { Level::from_raw(level.as_ptr(), &entries) }) {
            self.probe.biomes().answer_registry(&level, &plan.biome_requests, &mut self.sink);
        }
        if let Some(region) = client.region() {
            self.probe_plan(&entries, &region, &plan);
        }
        self.cursor.apply(plan.cursor, &mut self.window);
        self.plan = plan;
    }

    // a block of the registered source changed type: the minimap may ask for that one column. Game thread
    #[allow(unsafe_code)]
    pub fn block_changed(&mut self, change: &BlockChange) {
        if !self.in_world {
            return;
        }
        let entries = Arc::clone(&self.entries);
        if Self::same_type(&entries, change) {
            return;
        }
        // SAFETY: the game passes the live BlockSource that fired the change
        let Some(source) = (unsafe { ListenerSource::from_raw(change.source, &entries) }) else { return };
        let Some(dimension) = source.dimension_id() else { return };
        let ProbeRequest::Column { pos, index } = self.sink.block_changed(change.pos.x, change.pos.z, dimension) else {
            return;
        };
        // SAFETY: as above
        let Some(region) = (unsafe { BlockSource::from_raw(change.source, &entries) }) else { return };
        if let Some(sample) = self.probe.probe_column(&entries, &region, pos, index, &self.sink) {
            self.sink.submit_column(pos, index, sample);
        }
    }

    // stops listening, frees the listener once no source holds it, releases a cursor clip, and forgets the world,
    // so a tracker made by the next enable enters it again with a new listener. Off the game thread with a source
    // attached, the listener and its table stay allocated: the game thread may be about to call it through the
    // listener list of the source, and a leak of about 90 bytes is the safe side
    #[allow(unsafe_code)]
    pub fn reset(&mut self, on_game_thread: bool) {
        let attached = self.listener.as_ref().is_some_and(BlockListener::is_attached);
        self.detach_listener();
        if !on_game_thread && attached {
            if self.listener.take().is_some() {
                curia::info!(
                    "disable ran off the game thread while a source held the block listener; the listener stays allocated"
                );
            }
        } else if let (Some(listener), Some(heap)) = (self.listener.take(), self.heap.as_ref()) {
            // SAFETY: the listener was allocated on `heap`; it was just removed from its source, or its source was
            // destroyed, and reset runs after the hooks are gone
            if unsafe { listener.release(heap) }.is_err() {
                curia::warn!("the block listener is still held by a source and is not freed");
            }
        }
        self.cursor.release();
        self.in_world = false;
        self.dimension = -1;
    }

    fn enter(&mut self, entries: &GameEntries, client: &ClientInstance<'_>, player: &Actor<'_>, dimension: i32) {
        #[allow(unsafe_code)]
        // SAFETY: the actor's level is live while the actor is
        let level = player.level().and_then(|level| unsafe { Level::from_raw(level.as_ptr(), entries) });
        let identity = self.identity.read(entries, client, level.as_ref(), self.storage.current());
        if let Err(error) = self.sink.enter_world(&identity, dimension) {
            curia::error!("world not entered", { error: error.to_string() });
        }
        self.attach_listener(entries, player);
        self.in_world = true;
        self.dimension = dimension;
    }

    fn leave(&mut self) {
        if !self.in_world {
            return;
        }
        self.detach_listener();
        self.sink.leave_world();
        self.in_world = false;
        self.dimension = -1;
    }

    // out of a world the minimap still ticks with no player: it drains the manager's events and gives its leases
    // back
    fn tick_without_player(&mut self, client: &ClientInstance<'_>) {
        let input = TickInput {
            foreground: self.window.foreground(),
            cursor_hidden: self.cursor.cursor_hidden(),
            mouse_grabbed: client.mouse_grabbed(),
            ..TickInput::default()
        };
        let mut plan = std::mem::take(&mut self.plan);
        self.sink.tick(&input, &mut plan);
        self.cursor.apply(plan.cursor, &mut self.window);
        self.plan = plan;
    }

    // probes the candidates in order and stops reading after `limit` loaded chunks
    fn probe_plan(&mut self, entries: &GameEntries, region: &BlockSource<'_>, plan: &ScanPlan) {
        let mut loaded = 0;
        for &pos in &plan.candidates {
            if loaded >= plan.limit {
                self.sink.submit_chunk(pos, SubmitStatus::NotProbed, &self.staging);
                continue;
            }
            *self.staging = ChunkProbe::zeroed();
            let status = self.probe.probe_chunk(entries, region, pos, &self.sink, &mut self.staging);
            if status == SubmitStatus::Loaded {
                loaded += 1;
                self.probe.name_biomes(entries, region, pos, &self.staging, &plan.biome_requests, &mut self.sink);
            }
            self.sink.submit_chunk(pos, status, &self.staging);
        }
    }

    fn listener_lost(&self) -> bool {
        self.listener.as_ref().is_some_and(|listener| !listener.is_attached())
    }

    #[allow(unsafe_code)]
    fn attach_listener(&mut self, entries: &GameEntries, player: &Actor<'_>) {
        if self.listener.is_none()
            && let (Some((base, slots)), Some(heap)) = (self.listener_slots, self.heap.as_ref())
        {
            // SAFETY: `base` is the resolved BlockSourceListener table of 9 code pointers and `heap` the game heap
            match unsafe { BlockListener::new(base, slots, self.hub, heap) } {
                Ok(listener) => self.listener = Some(listener),
                Err(error) => {
                    curia::warn!("block changes are off", { error: error.to_string() });
                    self.listener_slots = None;
                }
            }
        }
        let Some(listener) = self.listener.as_ref() else { return };
        match player.block_source() {
            // SAFETY: the game returned the player's live dimension BlockSource
            Some(source) => unsafe { listener.attach(entries, source.as_ptr()) },
            None => listener.detach(entries),
        }
    }

    fn detach_listener(&mut self) {
        if let Some(listener) = self.listener.as_ref() {
            listener.detach(&self.entries);
        }
    }

    // the new and old blocks have the same type name, so the map colour cannot change
    #[allow(unsafe_code)]
    fn same_type(entries: &GameEntries, change: &BlockChange) -> bool {
        // SAFETY: the game passes live Block references for the call
        let block = unsafe { amml_game::Block::from_raw(change.block, entries) };
        // SAFETY: as above
        let old = unsafe { amml_game::Block::from_raw(change.old_block, entries) };
        let name = block.as_ref().and_then(|block| block.type_name()).map(|name| name.as_bytes());
        let old_name = old.as_ref().and_then(|old| old.type_name()).map(|name| name.as_bytes());
        name == old_name
    }
}
