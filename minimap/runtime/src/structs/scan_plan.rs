use minimap_core::structs::pos::ChunkPos;

use crate::structs::CursorAction;

// the host probes candidates in order and stops after `limit` loaded chunks
#[derive(Debug, Clone, Default)]
pub struct ScanPlan {
    pub candidates: Vec<ChunkPos>,
    pub limit: usize,
    pub camera_height: i32,
    pub transparent_water: bool,
    pub cursor: CursorAction,
    // game biome IDs whose internal names the host answers with the biome-name lookup; refilled each tick
    pub biome_requests: Vec<u16>,
}
