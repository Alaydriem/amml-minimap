use std::time::{Duration, Instant};

use minimap_core::services::telemetry::SampleWindow;

use crate::structs::{RuntimeStats, TelemetryRow};

// collects frame, tick and probe timings between telemetry rows
pub struct TelemetryRecorder {
    started: Instant,
    last_row: Instant,
    interval: Duration,
    render_ms: SampleWindow,
    ui_build_ms: SampleWindow,
    tick_ms: SampleWindow,
    upload_ms: SampleWindow,
    upload_bytes: u64,
    // one tick's work: from the tick call to the last chunk the host submitted after it, so the
    // host's probing is included
    tick_started: Option<Instant>,
    tick_end: Option<Instant>,
    last_probe: Option<Instant>,
    probe_us_total: f64,
    chunks_probed: u64,
    working_set_bytes: u64,
    private_bytes: u64,
    last_bakes: (u64, u64),
    last_disk_bytes: u64,
}

impl TelemetryRecorder {
    pub const CAPACITY: usize = 16_384;
    const MIB: f32 = 1024.0 * 1024.0;

    pub fn new(interval: Duration, now: Instant) -> Self {
        Self {
            started: now,
            last_row: now,
            interval: interval.max(Duration::from_millis(100)),
            render_ms: SampleWindow::new(Self::CAPACITY),
            ui_build_ms: SampleWindow::new(Self::CAPACITY),
            tick_ms: SampleWindow::new(Self::CAPACITY),
            upload_ms: SampleWindow::new(Self::CAPACITY),
            upload_bytes: 0,
            tick_started: None,
            tick_end: None,
            last_probe: None,
            probe_us_total: 0.0,
            chunks_probed: 0,
            working_set_bytes: 0,
            private_bytes: 0,
            last_bakes: (0, 0),
            last_disk_bytes: 0,
        }
    }

    // one layer pass: its own paint time, and the time and bytes of its texture uploads
    pub fn on_paint(&mut self, paint_us: u32, upload_us: u32, upload_bytes: u64) {
        if paint_us > 0 {
            self.ui_build_ms.push(paint_us as f32 / 1000.0);
        }
        if upload_us > 0 {
            self.upload_ms.push(upload_us as f32 / 1000.0);
        }
        self.upload_bytes += upload_bytes;
    }

    // the whole of this mod's work in one manager frame, as the frame handler timed it
    pub fn on_render(&mut self, render_us: u32) {
        if render_us > 0 {
            self.render_ms.push(render_us as f32 / 1000.0);
        }
    }

    // process memory as the host sampled it; zero means not sampled
    pub fn on_memory(&mut self, working_set_bytes: u64, private_bytes: u64) {
        if working_set_bytes > 0 {
            self.working_set_bytes = working_set_bytes;
            self.private_bytes = private_bytes;
        }
    }

    // the runtime's tick call ran from `started` to `ended`; the host probes after it
    pub fn on_tick(&mut self, started: Instant, ended: Instant) {
        self.close_tick();
        self.tick_started = Some(started);
        self.tick_end = Some(ended);
    }

    // a chunk the host probed (submitted as loaded)
    pub fn on_probe(&mut self, now: Instant) {
        self.chunks_probed += 1;
        self.last_probe = Some(now);
    }

    pub fn due(&self, now: Instant) -> bool {
        now.duration_since(self.last_row) >= self.interval
    }

    // `bakes` is (count, nanoseconds) since start
    pub fn take_row(&mut self, now: Instant, stats: &RuntimeStats, bakes: (u64, u64)) -> TelemetryRow {
        self.close_tick();
        let (count, nanos) = (bakes.0.saturating_sub(self.last_bakes.0), bakes.1.saturating_sub(self.last_bakes.1));
        // a new world starts a new store, whose total starts again at zero
        let disk = match stats.disk_written_bytes.checked_sub(self.last_disk_bytes) {
            Some(diff) => diff,
            None => stats.disk_written_bytes,
        };
        let row = TelemetryRow {
            t_ms: now.duration_since(self.started).as_millis() as u64,
            render_ms_p50: self.render_ms.quantile(0.5),
            render_ms_p99: self.render_ms.quantile(0.99),
            ui_build_ms_p50: self.ui_build_ms.quantile(0.5),
            ui_build_ms_p99: self.ui_build_ms.quantile(0.99),
            tick_ms_p50: self.tick_ms.quantile(0.5),
            tick_ms_p99: self.tick_ms.quantile(0.99),
            chunks_probed: self.chunks_probed,
            probe_us_mean: if self.chunks_probed == 0 {
                0.0
            } else {
                (self.probe_us_total / self.chunks_probed as f64) as f32
            },
            bake_us_mean: if count == 0 { 0.0 } else { nanos as f32 / count as f32 / 1000.0 },
            bake_queue: stats.bake_queue,
            surfaces_resident: stats.surfaces_resident,
            tiles_resident: stats.tiles_resident,
            tile_overrun: stats.tile_overrun,
            gpu_textures: stats.gpu_textures,
            working_set_mb: self.working_set_bytes as f32 / Self::MIB,
            private_mb: self.private_bytes as f32 / Self::MIB,
            // the manager tessellates now, so the mod has no vertex count
            ui_vtx: 0,
            disk_write_kb: disk as f32 / 1024.0,
            upload_ms_p50: self.upload_ms.quantile(0.5),
            upload_ms_p99: self.upload_ms.quantile(0.99),
            upload_kb: self.upload_bytes as f32 / 1024.0,
            gpu_texture_mb: stats.gpu_texture_mb,
            gpu_texture_budget_mb: stats.gpu_texture_budget_mb,
        };
        self.render_ms.clear();
        self.ui_build_ms.clear();
        self.tick_ms.clear();
        self.upload_ms.clear();
        self.upload_bytes = 0;
        self.probe_us_total = 0.0;
        self.chunks_probed = 0;
        self.last_bakes = bakes;
        self.last_disk_bytes = stats.disk_written_bytes;
        self.last_row = now;
        row
    }

    fn close_tick(&mut self) {
        let (Some(started), Some(ended)) = (self.tick_started.take(), self.tick_end.take()) else { return };
        let last_probe = self.last_probe.take().filter(|probe| *probe > ended);
        if let Some(probe) = last_probe {
            self.probe_us_total += probe.duration_since(ended).as_secs_f64() * 1_000_000.0;
        }
        let end = last_probe.unwrap_or(ended);
        self.tick_ms.push(end.duration_since(started).as_secs_f32() * 1000.0);
    }
}
