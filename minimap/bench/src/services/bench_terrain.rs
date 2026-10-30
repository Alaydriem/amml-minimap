use minimap_core::services::memory::ChunkStore;
use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface, ColumnSample};
use minimap_core::structs::pos::ChunkPos;

// deterministic terrain for benches: rolling hills, steps, and lakes below sea level.
// Integer maths only, so every machine builds the same chunks
pub struct BenchTerrain;

impl BenchTerrain {
    pub const SEA_LEVEL: i16 = 62;

    pub fn height(x: i32, z: i32) -> i16 {
        let tri = |v: i32, period: i32, amp: i32| (v.rem_euclid(period) - period / 2).abs() * 2 * amp / period;
        let step = if (x.div_euclid(80) + z.div_euclid(64)).rem_euclid(3) == 0 { 14 } else { 0 };
        (52 + tri(x, 97, 20) + tri(z, 71, 16) + tri(x + z, 29, 6) + step) as i16
    }

    pub fn column(x: i32, z: i32) -> ColumnSample {
        let h = Self::height(x, z);
        if h < Self::SEA_LEVEL {
            let depth = (Self::SEA_LEVEL - h) as u8;
            let sand = Rgba::new(196, 180, 120, 255).packed();
            let water = Rgba::new(40, 80, 200, depth).packed();
            return ColumnSample::new(sand, water, Self::SEA_LEVEL, h, false, 0);
        }
        let shade = (h - 40).clamp(0, 80) as u8;
        ColumnSample::new(Rgba::new(60 + shade, 140, 50 + shade / 2, 255).packed(), 0, h, h, false, 0)
    }

    // one chunk as the column probe submits it
    pub fn probe(pos: ChunkPos) -> ChunkSurface {
        let mut surface = ChunkSurface::empty();
        for lz in 0..16 {
            for lx in 0..16 {
                let (x, z) = (pos.min_block_x() + lx as i32, pos.min_block_z() + lz as i32);
                surface.write_column(ChunkSurface::index(lx, lz), Self::column(x, z));
            }
        }
        surface
    }

    // game biome 1 where x mod 64 is below 32, else 4
    pub fn biome_id(x: i32, _z: i32) -> u16 {
        if x.rem_euclid(64) < 32 { 1 } else { 4 }
    }

    // one chunk as the column probe submits it, with the raw biome ID of every column
    pub fn probe_with_biomes(pos: ChunkPos) -> ChunkProbe {
        let mut ids = [0u16; 256];
        for lz in 0..16 {
            for lx in 0..16 {
                ids[ChunkSurface::index(lx, lz)] =
                    Self::biome_id(pos.min_block_x() + lx as i32, pos.min_block_z() + lz as i32);
            }
        }
        ChunkProbe::new(Self::probe(pos), ids)
    }

    // one chunk as the chunk store keeps it
    pub fn stored(pos: ChunkPos) -> ChunkSurface {
        ChunkSurface::empty().merge(&Self::probe(pos)).surface
    }

    // every chunk of the square of `radius` chunks around `centre`
    pub fn chunks(centre: ChunkPos, radius: i32) -> Vec<(ChunkPos, ChunkSurface)> {
        let mut out = Vec::with_capacity(((2 * radius + 1) * (2 * radius + 1)) as usize);
        for z in centre.z - radius..=centre.z + radius {
            for x in centre.x - radius..=centre.x + radius {
                let pos = ChunkPos::new(x, z, centre.dim);
                out.push((pos, Self::stored(pos)));
            }
        }
        out
    }

    // a chunk store that holds `chunks`, with room for all of them
    pub fn store(chunks: &[(ChunkPos, ChunkSurface)]) -> ChunkStore {
        let mut store = ChunkStore::new(chunks.len().max(1));
        let mut evicted = Vec::new();
        let centre = chunks.first().map_or(ChunkPos::new(0, 0, 0), |(pos, _)| *pos);
        for (pos, surface) in chunks {
            store.restore(*pos, surface, centre, &mut evicted);
        }
        store
    }
}
