use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64};
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock};

use crossbeam_channel::{Receiver, Sender};
use minimap_core::services::biome::{BiomeKeyTable, HoverBiomeCache};
use minimap_core::services::memory::{ChunkStore, KnownEmptySet, TileStore};
use minimap_core::services::ui::MapUi;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::bake::BakeSettings;
use minimap_core::structs::export::ViewRegions;
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::view::{PlayerSnapshot, ViewState};

use crate::structs::BakeTiming;

// state read by the game thread, the bake workers and the render thread
pub struct Shared {
    // written only by the game thread, which never waits for it
    pub chunks: RwLock<ChunkStore>,
    pub tiles: Mutex<TileStore>,
    pub known_empty: Mutex<KnownEmptySet>,
    pub player: Mutex<PlayerSnapshot>,
    pub view: Mutex<ViewState>,
    // the current world's waypoints; changed by the game thread, copied by the render thread
    pub waypoints: Mutex<WaypointStore>,
    // the world map's menu, dialog and message line; lock it before `waypoints` when both are needed
    pub ui: Mutex<MapUi>,
    // the world's biome names by key; the game thread fills it, the render thread only try-locks it
    pub biome_keys: Mutex<BiomeKeyTable>,
    // biome palettes of regions read from disk for the hover info; Runtime::new sizes it from config
    pub hover_biomes: Mutex<HoverBiomeCache>,
    // regions the hover info wants biomes for; the game thread forwards them to the store
    pub biome_requests: (Sender<RegionPos>, Receiver<RegionPos>),
    pub settings: BakeSettings,
    pub settings_hash: u64,
    // regions the render thread wants loaded; the game thread forwards them to the store
    pub tile_requests: (Sender<RegionPos>, Receiver<RegionPos>),
    // bumped on every world entry and exit, so the render thread drops its textures
    pub world_epoch: AtomicU64,
    // the world map asked for an explored-area export; the game thread starts it
    pub area_export: AtomicBool,
    // the world map asked to export the regions on screen; the game thread starts it
    pub view_export: Mutex<Option<ViewRegions>>,
    // the map layer shown: a dimension id, or a cave band's layer id; written by the game thread
    pub layer: AtomicI32,
    // a world map text field had the keyboard at the end of the last overlay pass; written by the render thread
    pub typing: AtomicBool,
    pub bake_timing: BakeTiming,
}

impl Shared {
    pub const TILE_REQUEST_CAPACITY: usize = 256;
    pub const BIOME_REQUEST_CAPACITY: usize = 16;

    pub fn new(
        chunks: ChunkStore,
        tiles: TileStore,
        known_empty: KnownEmptySet,
        settings: BakeSettings,
        view: ViewState,
        waypoints: WaypointStore,
        ui: MapUi,
    ) -> Self {
        Self {
            chunks: RwLock::new(chunks),
            tiles: Mutex::new(tiles),
            known_empty: Mutex::new(known_empty),
            player: Mutex::new(PlayerSnapshot::default()),
            view: Mutex::new(view),
            waypoints: Mutex::new(waypoints),
            ui: Mutex::new(ui),
            biome_keys: Mutex::new(BiomeKeyTable::new(BiomeKeyTable::CAPACITY)),
            hover_biomes: Mutex::new(HoverBiomeCache::new(1)),
            biome_requests: crossbeam_channel::bounded(Self::BIOME_REQUEST_CAPACITY),
            settings_hash: settings.settings_hash(),
            settings,
            tile_requests: crossbeam_channel::bounded(Self::TILE_REQUEST_CAPACITY),
            world_epoch: AtomicU64::new(0),
            area_export: AtomicBool::new(false),
            view_export: Mutex::new(None),
            layer: AtomicI32::new(0),
            typing: AtomicBool::new(false),
            bake_timing: BakeTiming::default(),
        }
    }

    // a panic on another thread must not take the map down with it
    pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
