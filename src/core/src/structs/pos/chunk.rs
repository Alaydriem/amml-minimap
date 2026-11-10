use crate::structs::pos::RegionPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
    pub dim: i32,
}

impl ChunkPos {
    pub const BLOCKS: i32 = 16;

    pub fn new(x: i32, z: i32, dim: i32) -> Self {
        Self { x, z, dim }
    }

    pub fn from_block(block_x: i32, block_z: i32, dim: i32) -> Self {
        Self::new(block_x.div_euclid(Self::BLOCKS), block_z.div_euclid(Self::BLOCKS), dim)
    }

    pub fn offset(&self, dx: i32, dz: i32) -> Self {
        Self::new(self.x + dx, self.z + dz, self.dim)
    }

    pub fn region(&self) -> RegionPos {
        RegionPos::new(self.x.div_euclid(RegionPos::CHUNKS), self.z.div_euclid(RegionPos::CHUNKS), self.dim)
    }

    pub fn local_in_region(&self) -> (usize, usize) {
        (self.x.rem_euclid(RegionPos::CHUNKS) as usize, self.z.rem_euclid(RegionPos::CHUNKS) as usize)
    }

    pub fn min_block_x(&self) -> i32 {
        self.x * Self::BLOCKS
    }

    pub fn min_block_z(&self) -> i32 {
        self.z * Self::BLOCKS
    }

    pub fn distance_squared(&self, other: &ChunkPos) -> i64 {
        let dx = i64::from(self.x - other.x);
        let dz = i64::from(self.z - other.z);
        dx * dx + dz * dz
    }
}
