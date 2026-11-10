use minimap_core::structs::chunk::{ChunkSurface, DirtyEpochs, ShadowMask};
use minimap_core::structs::pos::ChunkPos;

// one chunk as a worker baked it: the surface it used, the epochs it saw, what it produced
#[derive(Debug, Clone)]
pub struct BakedChunk {
    pub pos: ChunkPos,
    pub epochs: DirtyEpochs,
    pub mask: ShadowMask,
    pub pixels: [u32; 256],
    pub surface: Box<ChunkSurface>,
}
