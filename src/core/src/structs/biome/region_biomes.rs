use crate::structs::biome::{BiomeKey, BiomePalette};

// the biome palettes of the stored chunks of one region, read from disk for hover
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionBiomes {
    // bit lz * 16 + lx is set for every chunk with a palette
    present: [u64; 4],
    palettes: Box<[BiomePalette]>,
}

impl RegionBiomes {
    pub fn empty() -> Self {
        Self { present: [0; 4], palettes: vec![BiomePalette::empty(); 256].into_boxed_slice() }
    }

    pub fn set(&mut self, local: (usize, usize), palette: BiomePalette) {
        let index = local.1 * 16 + local.0;
        self.palettes[index] = palette;
        self.present[index / 64] |= 1 << (index % 64);
    }

    pub fn key(&self, local: (usize, usize), column: usize) -> Option<BiomeKey> {
        let index = local.1 * 16 + local.0;
        if (self.present[index / 64] >> (index % 64)) & 1 == 0 {
            return None;
        }
        self.palettes[index].key(column)
    }

    pub fn is_empty(&self) -> bool {
        self.present.iter().all(|word| *word == 0)
    }
}
