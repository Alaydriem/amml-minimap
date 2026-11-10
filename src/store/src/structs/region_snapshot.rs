use minimap_core::structs::biome::BiomePalette;
use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::tile::Tile;

use crate::structs::JournalRecord;

// everything stored for one region: the tile pixels and the surface of each stored chunk
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionSnapshot {
    pub region: RegionPos,
    pub settings_hash: u64,
    // the biome key table the palettes were keyed against
    pub table_id: u64,
    pub tile: Box<[u8]>,
    // one slot per chunk, local_z * 16 + local_x
    pub surfaces: Vec<Option<Box<ChunkSurface>>>,
}

impl RegionSnapshot {
    pub fn new(region: RegionPos, settings_hash: u64) -> Self {
        Self {
            region,
            settings_hash,
            table_id: 0,
            tile: vec![0u8; Tile::BYTES].into_boxed_slice(),
            surfaces: vec![None; 256],
        }
    }

    pub fn present_bits(&self) -> [u64; 4] {
        let mut bits = [0u64; 4];
        for (i, slot) in self.surfaces.iter().enumerate() {
            if slot.is_some() {
                bits[i / 64] |= 1 << (i % 64);
            }
        }
        bits
    }

    pub fn surface_count(&self) -> usize {
        self.surfaces.iter().filter(|s| s.is_some()).count()
    }

    pub fn has_pixels(&self) -> bool {
        self.tile.iter().skip(3).step_by(4).any(|alpha| *alpha != 0)
    }

    // any baked pixel inside the chunk's 16 x 16 patch of the tile
    pub fn chunk_has_pixels(&self, local_x: usize, local_z: usize) -> bool {
        (0..16).any(|z| {
            let row = ((local_z * 16 + z) * Tile::SIDE + local_x * 16) * 4;
            (0..16).any(|x| self.tile[row + x * 4 + 3] != 0)
        })
    }

    pub fn is_empty(&self) -> bool {
        self.surface_count() == 0 && !self.has_pixels()
    }

    // palettes keyed against another table would name the wrong biomes, so they are dropped; table 0
    // means the world's table is not known yet, and then nothing is dropped
    pub fn retain_biomes(&mut self, table_id: u64) {
        if table_id == 0 || self.table_id == table_id {
            return;
        }
        for surface in self.surfaces.iter_mut().flatten() {
            surface.biome = BiomePalette::empty();
        }
        self.table_id = table_id;
    }

    // lays a newer journal record over the snapshot; a palette from another table is dropped
    pub fn apply(&mut self, record: &JournalRecord) {
        let (lx, lz) = record.pos.local_in_region();
        let mut surface = record.surface;
        if record.table_id != self.table_id {
            surface.biome = BiomePalette::empty();
        }
        self.surfaces[lz * 16 + lx] = Some(Box::new(surface));
        let Some(patch) = &record.patch else { return };
        for z in 0..16 {
            for x in 0..16 {
                let i = ((lz * 16 + z) * Tile::SIDE + lx * 16 + x) * 4;
                self.tile[i..i + 4].copy_from_slice(&patch[z * 16 + x].to_le_bytes());
            }
        }
        self.settings_hash = record.settings_hash;
    }
}
