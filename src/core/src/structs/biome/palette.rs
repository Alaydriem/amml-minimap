use bytemuck::{Pod, Zeroable};

use crate::structs::biome::BiomeKey;

// the biome of every column of a chunk: up to 3 keys and a 2-bit slot index per column.
// Index 0 means unknown, so a zeroed palette reads as unknown everywhere. A chunk never holds many
// biomes at once, so a column past the third biome takes the chunk's first one
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct BiomePalette {
    // slot 0 is never used
    keys: [u16; 4],
    // column c is bits (c % 4) * 2 of byte c / 4
    index: [u8; 64],
}

const _: () = assert!(std::mem::size_of::<BiomePalette>() == BiomePalette::BYTES);

impl BiomePalette {
    pub const BYTES: usize = 72;
    pub const SLOTS: usize = 3;
    const COLUMNS: usize = 256;
    const FIRST: usize = 1;

    pub fn empty() -> Self {
        Self::zeroed()
    }

    pub fn key(&self, column: usize) -> Option<BiomeKey> {
        let slot = self.slot(column);
        (slot != 0).then(|| BiomeKey(self.keys[slot]))
    }

    // re-uses the slot that holds `key`, else takes a slot no column refers to; false when the
    // palette is full, and the column then takes the first biome
    pub fn set(&mut self, column: usize, key: Option<BiomeKey>) -> bool {
        self.write_slot(column, 0);
        let Some(BiomeKey(key)) = key else { return true };
        // a slot no column refers to may still hold `key`; re-using it is harmless
        if let Some(slot) = (1..=Self::SLOTS).find(|&s| self.keys[s] == key) {
            self.write_slot(column, slot);
            return true;
        }
        let used = self.used_slots();
        let Some(slot) = (1..=Self::SLOTS).find(|&s| !used[s]) else {
            self.write_slot(column, Self::FIRST);
            return false;
        };
        self.keys[slot] = key;
        self.write_slot(column, slot);
        true
    }

    // gives `to` the biome of `from`, without a search; for runs of columns with one biome
    pub fn copy_column(&mut self, from: usize, to: usize) {
        let slot = self.slot(from);
        self.write_slot(to, slot);
    }

    // how many slots at least one column refers to
    pub fn distinct(&self) -> usize {
        self.used_slots()[1..].iter().filter(|u| **u).count()
    }

    fn used_slots(&self) -> [bool; 4] {
        let mut used = [false; 4];
        for column in 0..Self::COLUMNS {
            used[self.slot(column)] = true;
        }
        used
    }

    fn slot(&self, column: usize) -> usize {
        usize::from((self.index[column / 4] >> ((column % 4) * 2)) & 0b11)
    }

    fn write_slot(&mut self, column: usize, slot: usize) {
        let shift = (column % 4) * 2;
        let byte = &mut self.index[column / 4];
        *byte = (*byte & !(0b11 << shift)) | ((slot as u8) << shift);
    }
}
