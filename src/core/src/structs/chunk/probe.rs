use bytemuck::{Pod, Zeroable};

use crate::structs::chunk::ChunkSurface;

// what the probe hands over for one chunk: the surface and the raw game biome ID of each column.
// Core turns the IDs into the surface's biome palette
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct ChunkProbe {
    pub surface: ChunkSurface,
    pub biome_ids: [u16; 256],
}

const _: () = assert!(std::mem::size_of::<ChunkProbe>() == 3776);

impl ChunkProbe {
    pub fn new(surface: ChunkSurface, biome_ids: [u16; 256]) -> Self {
        Self { surface, biome_ids }
    }
}
