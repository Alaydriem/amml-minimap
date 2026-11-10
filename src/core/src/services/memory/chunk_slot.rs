use crate::structs::chunk::{ChunkSurface, DirtyFlags, ShadowMask};
use crate::structs::pos::ChunkPos;

pub(crate) struct ChunkSlot {
    pub(crate) pos: ChunkPos,
    pub(crate) surface: ChunkSurface,
    pub(crate) mask: ShadowMask,
    pub(crate) last_scan: Option<u64>,
    pub(crate) dirty: DirtyFlags,
    pub(crate) journal_dirty: bool,
    pub(crate) journal_version: u64,
}

impl ChunkSlot {
    pub(crate) fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            surface: ChunkSurface::empty(),
            mask: ShadowMask::new(0),
            last_scan: None,
            dirty: DirtyFlags::new(),
            journal_dirty: false,
            journal_version: 0,
        }
    }
}
