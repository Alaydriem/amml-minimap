use minimap_core::config::TerrainConfig;
use minimap_core::structs::bake::BakeSettings;

// the bake settings a capture was recorded with, so a replay bakes the same way
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureHeader {
    pub settings_hash: u64,
    pub terrain: TerrainConfig,
}

impl CaptureHeader {
    pub fn new(terrain: &TerrainConfig) -> Self {
        Self { settings_hash: BakeSettings::new(terrain).settings_hash(), terrain: terrain.clone() }
    }
}
