use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::ShadowConfig;

// the camelCase aliases read the terrain block of captures recorded before the HCL config
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct TerrainConfig {
    #[serde(alias = "scanMaxChunksPerFrame")]
    pub scan_max_chunks_per_frame: i32,
    #[serde(alias = "scanRadius")]
    pub scan_radius: i32,
    pub enable: bool,
    #[serde(alias = "rescanIntervalFrames")]
    pub rescan_interval_frames: i32,
    #[serde(alias = "regionBakeThreshold")]
    pub region_bake_threshold: i32,
    #[serde(alias = "enableDiskCache")]
    pub enable_disk_cache: bool,
    #[serde(alias = "enableTransparentWater")]
    pub enable_transparent_water: bool,
    #[serde(alias = "waterOpacityPerDepth")]
    pub water_opacity_per_depth: f32,
    #[serde(alias = "waterMaxOpacity")]
    pub water_max_opacity: f32,
    pub shadow: ShadowConfig,
    // block name to the RGBA colour drawn for it in place of the game's map colour
    #[serde(alias = "blockColors")]
    pub block_colors: BTreeMap<String, [u8; 4]>,
    // biome name to the RGBA colour of a column with no blocks, in place of the game's map grass colour
    #[serde(alias = "biomeColorOverrides")]
    pub biome_color_overrides: BTreeMap<String, [u8; 4]>,
}

impl TerrainConfig {
    pub const MAX_BLOCK_COLORS: usize = 64;
    pub const MAX_BIOME_COLOR_OVERRIDES: usize = 64;
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            scan_max_chunks_per_frame: 32,
            scan_radius: 128,
            enable: true,
            rescan_interval_frames: 1200,
            region_bake_threshold: 64,
            enable_disk_cache: true,
            enable_transparent_water: true,
            water_opacity_per_depth: 0.1,
            water_max_opacity: 0.6,
            shadow: ShadowConfig::default(),
            block_colors: BTreeMap::from([("minecraft:lava".to_string(), [230, 110, 20, 255])]),
            // the End's void: near black with a purple tint, in place of the game's map grass green
            biome_color_overrides: BTreeMap::from([("the_end".to_string(), [16, 12, 26, 255])]),
        }
    }
}
