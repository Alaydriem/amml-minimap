use crate::structs::pos::ChunkPos;

pub trait HeightRanges {
    // (min, max) surface height; None for a chunk with no base data
    fn height_range(&self, pos: ChunkPos) -> Option<(i16, i16)>;
}
