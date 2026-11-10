use crate::structs::Rgba;
use crate::structs::chunk::{ChunkSurface, ColumnSample};

// what a cave band's probe results become before they are stored: a column with no open space under
// the band top (the probe returns it see-through) is rock, and every column is flat at the band
// floor, because the probe's heights are the surface's and would shade the cave by the hills above
pub struct CaveSurface {
    rock: u32,
}

impl CaveSurface {
    pub fn new(rock: [u8; 4]) -> Self {
        Self { rock: Rgba::new(rock[0], rock[1], rock[2], rock[3]).packed() }
    }

    pub fn prepare_chunk(&self, surface: &mut ChunkSurface, floor: i16) {
        for index in 0..ChunkSurface::COLUMNS {
            if surface.is_skipped(index) {
                continue;
            }
            if Rgba::from_packed(surface.surface[index]).a == 0 {
                surface.surface[index] = self.rock;
            }
            surface.height[index] = floor;
            surface.solid[index] = floor;
        }
    }

    pub fn prepare_column(&self, sample: ColumnSample, floor: i16) -> ColumnSample {
        let surface = if Rgba::from_packed(sample.surface).a == 0 { self.rock } else { sample.surface };
        ColumnSample { surface, height: floor, solid: floor, ..sample }
    }
}
