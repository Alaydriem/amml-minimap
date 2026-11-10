// one line of telemetry.csv
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TelemetryRow {
    pub t_ms: u64,
    pub render_ms_p50: f32,
    pub render_ms_p99: f32,
    pub ui_build_ms_p50: f32,
    pub ui_build_ms_p99: f32,
    pub tick_ms_p50: f32,
    pub tick_ms_p99: f32,
    pub chunks_probed: u64,
    pub probe_us_mean: f32,
    pub bake_us_mean: f32,
    pub bake_queue: usize,
    pub surfaces_resident: usize,
    pub tiles_resident: usize,
    pub tile_overrun: usize,
    pub gpu_textures: usize,
    pub working_set_mb: f32,
    pub private_mb: f32,
    pub ui_vtx: u32,
    pub disk_write_kb: f32,
    pub upload_ms_p50: f32,
    pub upload_ms_p99: f32,
    pub upload_kb: f32,
    pub gpu_texture_mb: usize,
    pub gpu_texture_budget_mb: usize,
}

impl TelemetryRow {
    pub const HEADER: &'static str = "t_ms,render_ms_p50,render_ms_p99,ui_build_ms_p50,ui_build_ms_p99,tick_ms_p50,tick_ms_p99,chunks_probed,\
probe_us_mean,bake_us_mean,bake_queue,surfaces_resident,tiles_resident,tile_overrun,gpu_textures,\
working_set_mb,private_mb,ui_vtx,disk_write_kb,upload_ms_p50,upload_ms_p99,upload_kb,gpu_texture_mb,gpu_texture_budget_mb";

    pub fn to_csv(&self) -> String {
        format!(
            "{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{:.1},{:.1},{},{},{},{},{},{:.1},{:.1},{},{:.1},{:.3},{:.3},{:.1},{},{}",
            self.t_ms,
            self.render_ms_p50,
            self.render_ms_p99,
            self.ui_build_ms_p50,
            self.ui_build_ms_p99,
            self.tick_ms_p50,
            self.tick_ms_p99,
            self.chunks_probed,
            self.probe_us_mean,
            self.bake_us_mean,
            self.bake_queue,
            self.surfaces_resident,
            self.tiles_resident,
            self.tile_overrun,
            self.gpu_textures,
            self.working_set_mb,
            self.private_mb,
            self.ui_vtx,
            self.disk_write_kb,
            self.upload_ms_p50,
            self.upload_ms_p99,
            self.upload_kb,
            self.gpu_texture_mb,
            self.gpu_texture_budget_mb,
        )
    }
}
