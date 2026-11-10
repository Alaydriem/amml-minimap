use std::collections::HashMap;

use minimap_core::structs::chunk::{ChunkSurface, ShadowMask};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::traits::{BakeEnvironment, HeightRanges, ScanHistory};

#[derive(Default)]
pub struct FakeEnvironment {
    pub surfaces: HashMap<ChunkPos, ChunkSurface>,
    pub masks: HashMap<ChunkPos, ShadowMask>,
    pub ranges: HashMap<ChunkPos, (i16, i16)>,
    pub scans: HashMap<ChunkPos, u64>,
}

impl FakeEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_surface(mut self, pos: ChunkPos, surface: ChunkSurface) -> Self {
        self.surfaces.insert(pos, surface);
        self
    }

    pub fn with_mask(mut self, pos: ChunkPos, mask: ShadowMask) -> Self {
        self.masks.insert(pos, mask);
        self
    }

    pub fn with_range(mut self, pos: ChunkPos, min: i16, max: i16) -> Self {
        self.ranges.insert(pos, (min, max));
        self
    }
}

impl BakeEnvironment for FakeEnvironment {
    fn surface(&self, pos: ChunkPos) -> Option<&ChunkSurface> {
        self.surfaces.get(&pos).filter(|s| s.has_base_data())
    }

    fn mask(&self, pos: ChunkPos) -> Option<&ShadowMask> {
        self.masks.get(&pos)
    }
}

impl HeightRanges for FakeEnvironment {
    fn height_range(&self, pos: ChunkPos) -> Option<(i16, i16)> {
        self.ranges.get(&pos).copied()
    }
}

impl ScanHistory for FakeEnvironment {
    fn last_scan(&self, pos: ChunkPos) -> Option<u64> {
        self.scans.get(&pos).copied()
    }
}
