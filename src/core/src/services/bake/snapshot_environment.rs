use std::collections::HashMap;

use crate::structs::chunk::{ChunkSurface, ShadowMask};
use crate::structs::pos::ChunkPos;
use crate::traits::BakeEnvironment;

// a private copy of the chunks a bake reads, so the bake runs without holding the shared lock;
// slots are reused between bakes
pub struct SnapshotEnvironment {
    surfaces: Vec<ChunkSurface>,
    masks: Vec<ShadowMask>,
    has_mask: Vec<bool>,
    index: HashMap<ChunkPos, usize>,
    used: usize,
}

impl SnapshotEnvironment {
    pub fn new() -> Self {
        Self { surfaces: Vec::new(), masks: Vec::new(), has_mask: Vec::new(), index: HashMap::new(), used: 0 }
    }

    pub fn len(&self) -> usize {
        self.used
    }

    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    pub fn clear(&mut self) {
        self.index.clear();
        self.used = 0;
    }

    // copies every chunk with base data in the inclusive chunk rectangle
    pub fn capture_area(&mut self, source: &impl BakeEnvironment, x: (i32, i32), z: (i32, i32), dim: i32) {
        for cz in z.0..=z.1 {
            for cx in x.0..=x.1 {
                self.capture(source, ChunkPos::new(cx, cz, dim));
            }
        }
    }

    pub fn capture(&mut self, source: &impl BakeEnvironment, pos: ChunkPos) {
        let Some(surface) = source.surface(pos) else { return };
        if self.index.contains_key(&pos) {
            return;
        }
        if self.used == self.surfaces.len() {
            self.surfaces.push(*surface);
            self.masks.push(ShadowMask::new(0));
            self.has_mask.push(false);
        } else {
            self.surfaces[self.used] = *surface;
        }
        match source.mask(pos) {
            Some(mask) => {
                self.masks[self.used].copy_from(mask);
                self.has_mask[self.used] = true;
            }
            None => self.has_mask[self.used] = false,
        }
        self.index.insert(pos, self.used);
        self.used += 1;
    }
}

impl Default for SnapshotEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl BakeEnvironment for SnapshotEnvironment {
    fn surface(&self, pos: ChunkPos) -> Option<&ChunkSurface> {
        self.index.get(&pos).map(|&i| &self.surfaces[i])
    }

    fn mask(&self, pos: ChunkPos) -> Option<&ShadowMask> {
        self.index.get(&pos).filter(|&&i| self.has_mask[i]).map(|&i| &self.masks[i])
    }
}
