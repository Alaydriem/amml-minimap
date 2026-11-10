use std::collections::HashMap;

use crate::structs::draw::{ResidencyDecision, TextureOp};
use crate::structs::pos::RegionPos;

// which region tiles have a GPU texture; bounded by a budget, idle textures are released
pub struct GpuResidency {
    entries: HashMap<RegionPos, (u32, u64, u64)>,
    free_ids: Vec<u32>,
    next_id: u32,
    budget: usize,
    idle_frames: u64,
    max_new_per_frame: usize,
    created_this_frame: usize,
    frame: u64,
}

impl GpuResidency {
    // texture id 1 belongs to the minimap; region textures start at 2
    pub const MINIMAP_TEXTURE: u32 = 1;
    const FIRST_REGION_TEXTURE: u32 = 2;
    const NO_VERSION: u64 = u64::MAX;
    // one 256 x 256 RGBA region tile
    pub const TILE_BYTES: usize = 256 * 256 * 4;

    pub fn new(budget: usize, idle_frames: u64, max_new_per_frame: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(budget),
            free_ids: Vec::with_capacity(budget),
            next_id: Self::FIRST_REGION_TEXTURE,
            budget: budget.max(1),
            idle_frames,
            max_new_per_frame,
            created_this_frame: 0,
            frame: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn resident_bytes(&self) -> usize {
        self.entries.len() * Self::TILE_BYTES
    }

    pub fn budget_bytes(&self) -> usize {
        self.budget * Self::TILE_BYTES
    }

    pub fn budget(&self) -> usize {
        self.budget
    }

    // releases the texture of every region whose tile is no longer in memory, drawn or not
    pub fn release_unloaded(&mut self, is_loaded: impl Fn(RegionPos) -> bool, releases: &mut Vec<TextureOp>) {
        let free_ids = &mut self.free_ids;
        self.entries.retain(|region, (tex, _, _)| {
            let keep = is_loaded(*region);
            if !keep {
                releases.push(TextureOp::Release { tex: *tex });
                free_ids.push(*tex);
            }
            keep
        });
    }

    pub fn begin_frame(&mut self, frame: u64) {
        self.frame = frame;
        self.created_this_frame = 0;
    }

    // tile_version is None when the tile is not in memory
    pub fn decide(
        &mut self,
        region: RegionPos,
        tile_version: Option<u64>,
        releases: &mut Vec<TextureOp>,
    ) -> ResidencyDecision {
        if let Some(entry) = self.entries.get_mut(&region) {
            entry.2 = self.frame;
            return match tile_version {
                Some(version) if version != entry.1 => ResidencyDecision::Upload { tex: entry.0, created: false },
                _ => ResidencyDecision::Draw(entry.0),
            };
        }
        if tile_version.is_none() || self.created_this_frame >= self.max_new_per_frame {
            return ResidencyDecision::Placeholder;
        }
        if self.entries.len() >= self.budget && !self.evict_least_recent(releases) {
            return ResidencyDecision::Placeholder;
        }
        let tex = self.free_ids.pop().unwrap_or_else(|| {
            self.next_id += 1;
            self.next_id - 1
        });
        self.entries.insert(region, (tex, Self::NO_VERSION, self.frame));
        self.created_this_frame += 1;
        ResidencyDecision::Upload { tex, created: true }
    }

    pub fn uploaded(&mut self, region: RegionPos, version: u64) {
        if let Some(entry) = self.entries.get_mut(&region) {
            entry.1 = version;
        }
    }

    pub fn release_idle(&mut self, releases: &mut Vec<TextureOp>) {
        let (frame, idle) = (self.frame, self.idle_frames);
        let free_ids = &mut self.free_ids;
        self.entries.retain(|_, (tex, _, last_used)| {
            let keep = frame.saturating_sub(*last_used) <= idle;
            if !keep {
                releases.push(TextureOp::Release { tex: *tex });
                free_ids.push(*tex);
            }
            keep
        });
    }

    pub fn release_all(&mut self, releases: &mut Vec<TextureOp>) {
        for (_, (tex, _, _)) in self.entries.drain() {
            releases.push(TextureOp::Release { tex });
            self.free_ids.push(tex);
        }
    }

    // a texture not used in the current frame makes room; false when every texture is in use
    fn evict_least_recent(&mut self, releases: &mut Vec<TextureOp>) -> bool {
        let victim = self
            .entries
            .iter()
            .filter(|(_, (_, _, last_used))| *last_used < self.frame)
            .min_by_key(|(_, (_, _, last_used))| *last_used)
            .map(|(region, (tex, _, _))| (*region, *tex));
        let Some((region, tex)) = victim else { return false };
        self.entries.remove(&region);
        self.free_ids.push(tex);
        releases.push(TextureOp::Release { tex });
        true
    }
}
