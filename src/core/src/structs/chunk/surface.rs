use bytemuck::{Pod, Zeroable};

use crate::structs::Rgba;
use crate::structs::biome::{BiomeKey, BiomePalette};
use crate::structs::chunk::{ChangeKinds, ColumnSample, SurfaceMerge};

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct ChunkSurface {
    pub surface: [u32; 256],
    pub water: [u32; 256],
    pub height: [i16; 256],
    pub solid: [i16; 256],
    pub skipped: [u64; 4],
    pub retry: [u64; 4],
    pub min_height: i16,
    pub max_height: i16,
    pub flags: u32,
    pub biome: BiomePalette,
    pub padding: [u8; 48],
}

const _: () = assert!(std::mem::size_of::<ChunkSurface>() == 3264);

impl ChunkSurface {
    pub const COLUMNS: usize = 256;
    pub const SIDE: usize = 16;
    pub const NO_HEIGHT: i16 = -128;
    pub const HAS_BASE_DATA: u32 = 1;

    // every column marked skipped: nothing has been written yet
    pub fn empty() -> Self {
        Self { height: [Self::NO_HEIGHT; 256], solid: [Self::NO_HEIGHT; 256], skipped: [u64::MAX; 4], ..Self::zeroed() }
    }

    pub fn index(x: usize, z: usize) -> usize {
        z * Self::SIDE + x
    }

    pub fn has_base_data(&self) -> bool {
        self.flags & Self::HAS_BASE_DATA != 0
    }

    pub fn height_range(&self) -> Option<(i16, i16)> {
        self.has_base_data().then_some((self.min_height, self.max_height))
    }

    pub fn is_skipped(&self, index: usize) -> bool {
        (self.skipped[index / 64] >> (index % 64)) & 1 == 1
    }

    pub fn is_retry(&self, index: usize) -> bool {
        (self.retry[index / 64] >> (index % 64)) & 1 == 1
    }

    pub fn any_skipped(&self) -> bool {
        self.skipped.iter().any(|w| *w != 0)
    }

    pub fn any_retry(&self) -> bool {
        self.retry.iter().any(|w| *w != 0)
    }

    pub fn set_skipped(&mut self, index: usize, skipped: bool) {
        let bit = 1u64 << (index % 64);
        if skipped {
            self.skipped[index / 64] |= bit;
        } else {
            self.skipped[index / 64] &= !bit;
        }
    }

    pub fn set_retry(&mut self, index: usize, retry: bool) {
        let bit = 1u64 << (index % 64);
        if retry {
            self.retry[index / 64] |= bit;
        } else {
            self.retry[index / 64] &= !bit;
        }
    }

    pub fn write_column(&mut self, index: usize, column: ColumnSample) {
        self.surface[index] = column.surface;
        self.water[index] = column.water;
        self.height[index] = column.height;
        self.solid[index] = column.solid;
        self.set_skipped(index, false);
        self.set_retry(index, column.retry);
    }

    pub fn surface_colour(&self, index: usize) -> Rgba {
        Rgba::from_packed(self.surface[index])
    }

    pub fn water_depth(&self, index: usize) -> u8 {
        (self.water[index] >> 24) as u8
    }

    pub fn water_colour(&self, index: usize) -> Rgba {
        Rgba::from_packed(self.water[index] | 0xFF00_0000)
    }

    // self is the stored surface; incoming is a fresh probe of the same chunk
    pub fn merge(&self, incoming: &ChunkSurface) -> SurfaceMerge {
        let had_data = self.has_base_data();
        let mut result = *self;
        // a first load takes the probed palette whole, and an equal palette changes nothing; only a
        // partial rescan compares column by column
        let column_biomes = had_data && self.biome != incoming.biome;
        let mut changes = ChangeKinds::NONE;
        let mut min = i16::MAX;
        let mut max = i16::MIN;
        let mut written = false;
        for i in 0..Self::COLUMNS {
            if incoming.is_skipped(i) {
                continue;
            }
            written = true;
            min = min.min(incoming.height[i]);
            max = max.max(incoming.height[i]);
            if result.surface[i] != incoming.surface[i] || result.water[i] != incoming.water[i] {
                changes = changes.union(ChangeKinds::COLOUR);
            }
            if result.height[i] != incoming.height[i] {
                changes = changes.union(ChangeKinds::HEIGHT);
            }
            if result.solid[i] != incoming.solid[i] {
                changes = changes.union(ChangeKinds::SOLID);
            }
            if column_biomes && result.merge_biome(i, incoming.biome.key(i)) {
                changes = changes.union(ChangeKinds::BIOME);
            }
            result.surface[i] = incoming.surface[i];
            result.water[i] = incoming.water[i];
            result.height[i] = incoming.height[i];
            result.solid[i] = incoming.solid[i];
            result.set_skipped(i, false);
        }
        result.retry = incoming.retry;
        if !had_data {
            result.biome = incoming.biome;
        }
        if written {
            result.min_height = min;
            result.max_height = max;
            result.flags |= Self::HAS_BASE_DATA;
        }
        let first_load = !had_data && written;
        SurfaceMerge {
            surface: result,
            changes: if first_load { ChangeKinds::ALL } else { changes },
            first_load,
            previous_range: self.height_range(),
            needs_retry: incoming.any_skipped() || incoming.any_retry(),
        }
    }

    // a single-column update; None when the chunk has no base data yet
    pub fn merge_column(&self, index: usize, column: ColumnSample, biome: Option<BiomeKey>) -> Option<SurfaceMerge> {
        if !self.has_base_data() {
            return None;
        }
        let mut result = *self;
        let mut changes = ChangeKinds::NONE;
        if result.surface[index] != column.surface || result.water[index] != column.water {
            changes = changes.union(ChangeKinds::COLOUR);
        }
        if result.height[index] != column.height {
            changes = changes.union(ChangeKinds::HEIGHT);
        }
        if result.solid[index] != column.solid {
            changes = changes.union(ChangeKinds::SOLID);
        }
        if result.merge_biome(index, biome) {
            changes = changes.union(ChangeKinds::BIOME);
        }
        result.write_column(index, column);
        result.min_height = result.min_height.min(column.height);
        result.max_height = result.max_height.max(column.height);
        Some(SurfaceMerge {
            surface: result,
            changes,
            first_load: false,
            previous_range: self.height_range(),
            needs_retry: column.retry,
        })
    }

    // a known incoming biome replaces the stored one; an unknown one keeps it. True when it changed
    fn merge_biome(&mut self, index: usize, incoming: Option<BiomeKey>) -> bool {
        match incoming {
            Some(key) if self.biome.key(index) != Some(key) => {
                self.biome.set(index, Some(key));
                true
            }
            _ => false,
        }
    }
}
