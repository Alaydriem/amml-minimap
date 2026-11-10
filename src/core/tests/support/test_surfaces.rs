use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::{ChunkSurface, ColumnSample};

pub struct TestSurfaces;

impl TestSurfaces {
    pub fn grey(level: u8) -> u32 {
        Rgba::new(level, level, level, 255).packed()
    }

    // a fully probed chunk: every column the same colour, height and solid height
    pub fn probe_flat(height: i16, colour: u32) -> ChunkSurface {
        let mut surface = ChunkSurface::empty();
        for i in 0..ChunkSurface::COLUMNS {
            surface.write_column(i, ColumnSample::new(colour, 0, height, height, false, 0));
        }
        surface
    }

    // a stored chunk (merged once, so it has base data)
    pub fn stored_flat(height: i16, colour: u32) -> ChunkSurface {
        ChunkSurface::empty().merge(&Self::probe_flat(height, colour)).surface
    }

    pub fn with_height(surface: ChunkSurface, x: usize, z: usize, height: i16) -> ChunkSurface {
        let mut out = surface;
        let i = ChunkSurface::index(x, z);
        out.height[i] = height;
        out.solid[i] = height;
        if out.has_base_data() {
            out.min_height = out.min_height.min(height);
            out.max_height = out.max_height.max(height);
        }
        out
    }
}
