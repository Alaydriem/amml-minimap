use minimap_core::config::{MiniMapConfig, TerrainConfig};
use minimap_core::services::draw::{ChunkGridPainter, SlimeOverlayPainter};
use minimap_core::services::memory::ChunkStore;
use minimap_core::structs::bake::BakeSettings;
use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::draw::{DrawCommand, DrawList, ExploredChunks};
use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_store::services::RegionCodec;
use minimap_store::structs::RegionSnapshot;

use crate::services::{BenchTerrain, OverlayScenes};

// memory and disk sizes and vertex counts that a size claim cites
pub struct FootprintReport;

impl FootprintReport {
    const BUDGET_BYTES: usize = 64 << 20;
    // enough frames for the slime cache to hold every region of a 1920 x 1080 view at the gate
    const WARM_FRAMES: usize = 16;

    pub fn rows() -> Vec<(&'static str, u64)> {
        let scale = BakeSettings::new(&TerrainConfig::default()).scale;
        vec![
            ("chunk_surface_bytes", std::mem::size_of::<ChunkSurface>() as u64),
            ("chunks_per_64_mib", ChunkStore::capacity_for_budget(Self::BUDGET_BYTES, scale) as u64),
            ("region_file_bytes", Self::region_file_bytes()),
            ("grid_vertices_gate", Self::grid_vertices(OverlayScenes::GATE_ZOOM)),
            ("grid_vertices_max_zoom", Self::grid_vertices(OverlayScenes::MAX_ZOOM)),
            ("slime_vertices_gate", Self::slime_vertices(OverlayScenes::GATE_ZOOM)),
            ("slime_vertices_max_zoom", Self::slime_vertices(OverlayScenes::MAX_ZOOM)),
        ]
    }

    // the vertices egui tessellates for the list's shapes and text
    pub fn vertices(draw: &DrawList) -> u64 {
        draw.commands
            .iter()
            .map(|command| match command {
                DrawCommand::Line { .. } | DrawCommand::RectFilled { .. } => 4,
                DrawCommand::TriangleFilled { .. } => 3,
                DrawCommand::Text { len, .. } => 4 * u64::from(*len),
                _ => 0,
            })
            .sum()
    }

    fn grid_vertices(zoom: f32) -> u64 {
        let mut draw = DrawList::new();
        ChunkGridPainter::new(&MiniMapConfig::default()).paint(
            &OverlayScenes::world_frame(zoom),
            Some((0, 0)),
            &ExploredChunks::everywhere(),
            &mut draw,
        );
        Self::vertices(&draw)
    }

    // with every mask of the view cached, as after the first frames
    fn slime_vertices(zoom: f32) -> u64 {
        let frame = OverlayScenes::world_frame(zoom);
        let mut painter = SlimeOverlayPainter::new(4096);
        let mut draw = DrawList::new();
        for _ in 0..Self::WARM_FRAMES {
            draw.clear();
            painter.paint(&frame, &ExploredChunks::everywhere(), &mut draw);
        }
        Self::vertices(&draw)
    }

    // an explored region of bench terrain with an opaque tile, as the store writes it
    fn region_file_bytes() -> u64 {
        let region = RegionPos::new(0, 0, 0);
        let mut snapshot = RegionSnapshot::new(region, 0);
        snapshot.tile.fill(0xFF);
        for local_z in 0..16 {
            for local_x in 0..16 {
                let surface = BenchTerrain::probe(ChunkPos::new(local_x, local_z, 0));
                snapshot.surfaces[(local_z * 16 + local_x) as usize] = Some(Box::new(surface));
            }
        }
        RegionCodec::encode(&snapshot).map_or(0, |bytes| bytes.len() as u64)
    }
}
