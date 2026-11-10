use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::ChunkPos;

// the latest state of one chunk; `patch` is its baked 16 x 16 pixels when it has been baked
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalRecord {
    pub pos: ChunkPos,
    pub settings_hash: u64,
    // the biome key table the surface's palette was keyed against; the store sets it on append
    pub table_id: u64,
    pub surface: ChunkSurface,
    pub patch: Option<[u32; 256]>,
}

impl JournalRecord {
    pub fn new(pos: ChunkPos, settings_hash: u64, surface: ChunkSurface, patch: Option<[u32; 256]>) -> Self {
        Self { pos, settings_hash, table_id: 0, surface, patch }
    }
}
