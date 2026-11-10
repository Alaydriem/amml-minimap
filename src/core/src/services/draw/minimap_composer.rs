use crate::services::memory::TileStore;
use crate::structs::chunk::ChunkSurface;
use crate::structs::pos::{ChunkPos, RegionPos};
use crate::structs::tile::Tile;
use crate::traits::BakeEnvironment;

// the minimap texture: one texel per block around the player, plus a one-texel margin
pub struct MinimapComposer {
    image: Vec<u8>,
    side: usize,
    origin: (i32, i32),
    dim: i32,
    versions: Vec<(RegionPos, Option<u64>)>,
    scratch: Vec<(RegionPos, Option<u64>)>,
    valid: bool,
}

impl MinimapComposer {
    pub fn new() -> Self {
        Self {
            image: Vec::new(),
            side: 0,
            origin: (0, 0),
            dim: 0,
            versions: Vec::new(),
            scratch: Vec::new(),
            valid: false,
        }
    }

    pub fn image(&self) -> &[u8] {
        &self.image
    }

    pub fn side(&self) -> usize {
        self.side
    }

    pub fn origin(&self) -> (i32, i32) {
        self.origin
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn side_for(radius_chunks: i32) -> usize {
        (2 * radius_chunks * 16 + 2) as usize
    }

    // rebuilds only when the centre block, the radius, the dimension or a covered tile changed
    pub fn update(
        &mut self,
        centre: (i32, i32),
        dim: i32,
        radius_chunks: i32,
        tiles: &TileStore,
        chunks: Option<&impl BakeEnvironment>,
    ) -> bool {
        let side = Self::side_for(radius_chunks);
        let half = radius_chunks * 16 + 1;
        let origin = (centre.0 - half, centre.1 - half);
        self.scratch.clear();
        let first = RegionPos::from_block(origin.0, origin.1, dim);
        let last = RegionPos::from_block(origin.0 + side as i32 - 1, origin.1 + side as i32 - 1, dim);
        for rz in first.z..=last.z {
            for rx in first.x..=last.x {
                let region = RegionPos::new(rx, rz, dim);
                self.scratch.push((region, tiles.peek(region).map(Tile::version)));
            }
        }
        let unchanged = self.valid
            && self.side == side
            && self.origin == origin
            && self.dim == dim
            && self.versions == self.scratch;
        if unchanged {
            return false;
        }
        self.side = side;
        self.origin = origin;
        self.dim = dim;
        std::mem::swap(&mut self.versions, &mut self.scratch);
        self.image.clear();
        self.image.resize(side * side * 4, 0);
        for row in 0..side {
            self.compose_row(row, tiles, chunks);
        }
        self.valid = true;
        true
    }

    fn compose_row(&mut self, row: usize, tiles: &TileStore, chunks: Option<&impl BakeEnvironment>) {
        let world_z = self.origin.1 + row as i32;
        let mut col = 0;
        while col < self.side {
            let world_x = self.origin.0 + col as i32;
            let span = (16 - world_x.rem_euclid(16) as usize).min(self.side - col);
            let chunk = ChunkPos::from_block(world_x, world_z, self.dim);
            let tile = tiles.peek(chunk.region());
            let surface = chunks.and_then(|c| c.surface(chunk));
            for k in 0..span {
                let (bx, bz) = (world_x + k as i32, world_z);
                let baked = tile.map_or(0, |t| t.pixel(bx.rem_euclid(256) as usize, bz.rem_euclid(256) as usize));
                let pixel = if baked >> 24 != 0 {
                    baked
                } else {
                    surface.map_or(0, |s| {
                        s.surface[ChunkSurface::index(bx.rem_euclid(16) as usize, bz.rem_euclid(16) as usize)]
                    })
                };
                let i = (row * self.side + col + k) * 4;
                self.image[i..i + 4].copy_from_slice(&pixel.to_le_bytes());
            }
            col += span;
        }
    }
}

impl Default for MinimapComposer {
    fn default() -> Self {
        Self::new()
    }
}
