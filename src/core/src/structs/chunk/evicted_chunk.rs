use crate::structs::chunk::ChunkSurface;
use crate::structs::pos::ChunkPos;

// a chunk dropped from memory before its latest surface reached the journal
#[derive(Debug, Clone, Copy)]
pub struct EvictedChunk {
    pub pos: ChunkPos,
    pub surface: ChunkSurface,
}
