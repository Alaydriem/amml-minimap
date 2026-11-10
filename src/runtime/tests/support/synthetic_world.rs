use std::path::{Path, PathBuf};

use minimap_core::config::Config;
use std::time::Duration;

use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface, ColumnSample};
use minimap_core::structs::draw::{MapLayer, TextureGrant};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::{LayerInput, RuntimePaths, ScanPlan, SubmitStatus, TickInput};
use minimap_ui::testing::{HarnessEvent, HarnessInput, PassOutput, UiHarness};

use crate::support::Painted;
use minimap_store::structs::WorldIdentity;

// a deterministic hilly world that answers probes the way the host does
pub struct SyntheticWorld {
    pub dir: tempfile::TempDir,
    // the config the next runtime starts with, and the texture grant the manager gives it
    pub config: Config,
    pub grant: TextureGrant,
    pub plan: ScanPlan,
    pub frame: Painted,
    // the manager's two layer contexts
    pub hud: UiHarness,
    pub overlay: UiHarness,
    hud_out: PassOutput,
    overlay_out: PassOutput,
    // seconds of frame time drawn so far
    pub time: f64,
    // the screen the next frames draw on, and whether the pointer is over the game window
    pub display: (f32, f32),
    pub foreground: bool,
    pub player: PlayerSnapshot,
    // answer the plan's biome name requests the way the host does
    pub answer_biomes: bool,
}

impl SyntheticWorld {
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
            config: Config::default(),
            grant: TextureGrant { bytes: 1024 * 1024 * 1024, count: 4096 },
            plan: ScanPlan::default(),
            frame: Painted::new(),
            hud: UiHarness::new(),
            overlay: UiHarness::new(),
            hud_out: PassOutput::new(),
            overlay_out: PassOutput::new(),
            time: 0.0,
            display: Self::DISPLAY,
            foreground: true,
            player: PlayerSnapshot { present: true, x: 8.5, y: 80.0, z: 8.5, yaw: 0.0, dim: 0 },
            answer_biomes: true,
        }
    }

    pub fn paths(&self) -> RuntimePaths {
        RuntimePaths { data_dir: self.dir.path().join("data") }
    }

    pub fn world_dir(&self) -> PathBuf {
        self.dir.path().join("world")
    }

    pub fn identity(&self) -> WorldIdentity {
        WorldIdentity::Local { world_dir: self.world_dir() }
    }

    pub fn runtime(&self) -> Runtime {
        let runtime = Runtime::new(self.config.clone(), self.paths(), self.grant).unwrap();
        runtime.enter_world(&self.identity(), self.player.dim).unwrap();
        runtime
    }

    pub fn height(x: i32, z: i32) -> i16 {
        64 + ((x.rem_euclid(23) as i16 - 11).abs() + (z.rem_euclid(17) as i16 - 8).abs())
    }

    // game biome 1 (plains) where x mod 64 is below 32, else 4 (forest)
    pub fn biome_id(x: i32) -> u16 {
        if x.rem_euclid(64) < 32 { 1 } else { 4 }
    }

    pub fn probe(pos: ChunkPos) -> ChunkProbe {
        let mut surface = ChunkSurface::empty();
        let mut biomes = [0u16; 256];
        for lz in 0..16 {
            for lx in 0..16 {
                let (x, z) = (pos.min_block_x() + lx as i32, pos.min_block_z() + lz as i32);
                let h = Self::height(x, z);
                let shade = (h - 40).clamp(0, 255) as u8 * 4;
                let colour = Rgba::new(shade, 160, 60, 255).packed();
                let index = ChunkSurface::index(lx, lz);
                surface.write_column(index, ColumnSample::new(colour, 0, h, h, false, Self::biome_id(x)));
                biomes[index] = Self::biome_id(x);
            }
        }
        surface.skipped = [0; 4];
        ChunkProbe::new(surface, biomes)
    }

    pub fn biome_name(id: u16) -> &'static str {
        match id {
            1 => "plains",
            4 => "forest",
            _ => "",
        }
    }

    // one game tick: probe up to the plan's limit, report the rest as not probed
    pub fn tick(&mut self, runtime: &Runtime) {
        let input = TickInput {
            player: self.player,
            foreground: true,
            cursor_hidden: true,
            dead: false,
            mouse_grabbed: true,
            ..TickInput::default()
        };
        runtime.tick(&input, &mut self.plan);
        if self.answer_biomes {
            for &id in &self.plan.biome_requests {
                runtime.biome_name(id, Self::biome_name(id));
            }
        }
        for (i, pos) in self.plan.candidates.clone().into_iter().enumerate() {
            if i < self.plan.limit {
                runtime.submit_chunk(pos, SubmitStatus::Loaded, &Self::probe(pos));
            } else {
                runtime.submit_chunk(pos, SubmitStatus::NotProbed, &ChunkProbe::new(ChunkSurface::empty(), [0; 256]));
            }
        }
    }

    pub const DISPLAY: (f32, f32) = (1280.0, 720.0);
    pub const CENTRE: (f32, f32) = (640.0, 360.0);
    const FRAME_SECONDS: f64 = 0.016;

    pub fn draw(&mut self, runtime: &Runtime) {
        self.paint(runtime, Self::CENTRE, false, false, &[]);
    }

    // one manager frame: the hud layer, then the overlay layer, which alone gets the keys and text
    pub fn paint(&mut self, runtime: &Runtime, cursor: (f32, f32), left: bool, right: bool, events: &[HarnessEvent]) {
        self.time += Self::FRAME_SECONDS;
        let time = self.time;
        let input = HarnessInput {
            time_s: time,
            right_down: right,
            foreground: self.foreground,
            ..HarnessInput::new(self.display, cursor, left, &[])
        };
        let display = self.display;
        let mut hud = minimap_runtime::structs::LayerReply::EMPTY;
        self.hud.run(&input, &mut self.hud_out, |ctx| {
            hud = runtime.paint_layer(ctx, &Self::layer(MapLayer::Hud, display, time))
        });
        let input = HarnessInput { events, ..input };
        let mut overlay = minimap_runtime::structs::LayerReply::EMPTY;
        self.overlay.run(&input, &mut self.overlay_out, |ctx| {
            overlay = runtime.paint_layer(ctx, &Self::layer(MapLayer::Overlay, display, time))
        });
        self.frame.hud = hud;
        self.frame.overlay = overlay;
        self.frame.combine(&self.hud_out, &self.overlay_out);
        let drew = if overlay.arrow_cursor { MapLayer::Overlay } else { MapLayer::Hud };
        self.frame.draw = runtime.with_last_paint(drew, |paint| paint.draw.clone());
        self.frame.textures.clear();
        for layer in [MapLayer::Hud, MapLayer::Overlay] {
            runtime.with_last_paint(layer, |paint| self.frame.textures.extend_from_slice(&paint.stage.ops));
        }
    }

    fn layer(layer: MapLayer, display: (f32, f32), time_secs: f64) -> LayerInput {
        LayerInput { layer, width: display.0 as u32, height: display.1 as u32, pixels_per_point: 1.0, time_secs }
    }

    // ticks and draws until `done` holds or the step limit is reached; true when it held
    pub fn run_until(&mut self, runtime: &Runtime, steps: usize, done: impl Fn(&Runtime, &Painted) -> bool) -> bool {
        for _ in 0..steps {
            self.tick(runtime);
            self.draw(runtime);
            if done(runtime, &self.frame) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        false
    }

    pub fn exists(path: &Path) -> bool {
        path.exists()
    }
}
