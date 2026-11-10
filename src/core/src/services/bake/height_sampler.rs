use crate::structs::chunk::ChunkSurface;
use crate::structs::pos::ChunkPos;
use crate::traits::BakeEnvironment;

// surface height at any block, reading the chunk being baked from `center`
pub struct HeightSampler<'a, E: BakeEnvironment> {
    env: &'a E,
    center_pos: ChunkPos,
    center: &'a ChunkSurface,
    cached_pos: Option<ChunkPos>,
    cached: Option<&'a ChunkSurface>,
}

impl<'a, E: BakeEnvironment> HeightSampler<'a, E> {
    // height of a block in a chunk that is not loaded
    pub const MISSING: i32 = -65;

    pub fn new(env: &'a E, center_pos: ChunkPos, center: &'a ChunkSurface) -> Self {
        Self { env, center_pos, center, cached_pos: None, cached: None }
    }

    pub fn height_at(&mut self, block_x: i32, block_z: i32) -> i32 {
        let chunk = ChunkPos::from_block(block_x, block_z, self.center_pos.dim);
        let surface = if chunk == self.center_pos {
            Some(self.center)
        } else if self.cached_pos == Some(chunk) {
            self.cached
        } else {
            self.cached_pos = Some(chunk);
            self.cached = self.env.surface(chunk);
            self.cached
        };
        match surface {
            Some(s) => {
                let lx = block_x.rem_euclid(16) as usize;
                let lz = block_z.rem_euclid(16) as usize;
                i32::from(s.height[ChunkSurface::index(lx, lz)])
            }
            None => Self::MISSING,
        }
    }
}
