use crate::structs::pos::{ChunkPos, RegionPos};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BakeJob {
    Region(RegionPos),
    Chunk { pos: ChunkPos, soft_only: bool },
}
