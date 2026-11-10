use crate::structs::pos::ChunkPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionPos {
    pub x: i32,
    pub z: i32,
    pub dim: i32,
}

impl RegionPos {
    pub const CHUNKS: i32 = 16;
    pub const BLOCKS: i32 = 256;

    pub fn new(x: i32, z: i32, dim: i32) -> Self {
        Self { x, z, dim }
    }

    pub fn from_block(block_x: i32, block_z: i32, dim: i32) -> Self {
        Self::new(block_x.div_euclid(Self::BLOCKS), block_z.div_euclid(Self::BLOCKS), dim)
    }

    pub fn chunk(&self, local_x: usize, local_z: usize) -> ChunkPos {
        ChunkPos::new(self.x * Self::CHUNKS + local_x as i32, self.z * Self::CHUNKS + local_z as i32, self.dim)
    }

    pub fn contains(&self, chunk: &ChunkPos) -> bool {
        chunk.region() == *self
    }
}
