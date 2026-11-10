pub mod bake;
pub mod cave_mode;
pub mod clamp;
pub mod debug;
pub mod export;
pub mod keys;
pub mod memory;
pub mod merge;
pub mod mini_map;
pub mod shadow;
pub mod smooth_camera;
pub mod store;
pub mod terrain;
pub mod waypoints;
pub mod world_map;

pub use bake::BakeConfig;
pub use cave_mode::CaveModeConfig;
pub use clamp::ConfigClamp;
pub use debug::DebugConfig;
pub use export::ExportConfig;
pub use keys::KeysConfig;
pub use memory::MemoryConfig;
pub use merge::ConfigMerge;
pub use mini_map::MiniMapConfig;
pub use shadow::ShadowConfig;
pub use smooth_camera::SmoothCameraConfig;
pub use store::StoreConfig;
pub use terrain::TerrainConfig;
pub use waypoints::WaypointsConfig;
pub use world_map::WorldMapConfig;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Config {
    pub font_size: f32,
    pub mini_map: MiniMapConfig,
    pub smooth_camera: SmoothCameraConfig,
    pub terrain: TerrainConfig,
    pub world_map: WorldMapConfig,
    pub memory: MemoryConfig,
    pub bake: BakeConfig,
    pub store: StoreConfig,
    pub debug: DebugConfig,
    pub waypoints: WaypointsConfig,
    pub export: ExportConfig,
    pub cave_mode: CaveModeConfig,
    pub keys: KeysConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_size: 18.0,
            mini_map: MiniMapConfig::default(),
            smooth_camera: SmoothCameraConfig::default(),
            terrain: TerrainConfig::default(),
            world_map: WorldMapConfig::default(),
            memory: MemoryConfig::default(),
            bake: BakeConfig::default(),
            store: StoreConfig::default(),
            debug: DebugConfig::default(),
            waypoints: WaypointsConfig::default(),
            export: ExportConfig::default(),
            cave_mode: CaveModeConfig::default(),
            keys: KeysConfig::default(),
        }
    }
}
