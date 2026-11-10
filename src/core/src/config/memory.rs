use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct MemoryConfig {
    pub chunk_surface_budget_mb: i32,
    pub tile_budget_mb: i32,
    pub tile_overrun_percent: i32,
    // map tiles on the GPU, before the manager's texture grant caps it
    pub gpu_texture_budget_mb: i32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self { chunk_surface_budget_mb: 64, tile_budget_mb: 256, tile_overrun_percent: 25, gpu_texture_budget_mb: 496 }
    }
}
