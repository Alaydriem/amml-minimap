use crate::structs::chunk::{ChunkSurface, ShadowMask};
use crate::structs::pos::ChunkPos;

// read access to the chunks around the one being baked
pub trait BakeEnvironment {
    // only chunks that have base data
    fn surface(&self, pos: ChunkPos) -> Option<&ChunkSurface>;

    fn mask(&self, pos: ChunkPos) -> Option<&ShadowMask>;
}
