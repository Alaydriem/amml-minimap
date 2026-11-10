// counters for tests and telemetry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RuntimeStats {
    pub in_world: bool,
    pub ticks: u64,
    pub frames: u64,
    pub chunks_submitted: u64,
    pub bakes_completed: u64,
    pub tiles_loaded: u64,
    pub surfaces_resident: usize,
    pub surface_capacity: usize,
    pub tiles_resident: usize,
    pub tile_capacity: usize,
    pub tile_overrun: usize,
    pub gpu_textures: usize,
    // region tiles on the GPU, and the budget the setting and the manager's grant allow them
    pub gpu_texture_mb: usize,
    pub gpu_texture_budget_mb: usize,
    pub bake_queue: usize,
    pub outbox: usize,
    pub deferred: usize,
    pub disk_written_bytes: u64,
}
