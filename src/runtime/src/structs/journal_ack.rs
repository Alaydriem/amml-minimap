use minimap_core::structs::pos::{ChunkPos, RegionPos};

// what a journal append covered, so its acknowledgement clears only marks it can vouch for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JournalAck {
    pub pos: ChunkPos,
    pub journal_version: u64,
    // the tile version after this chunk's pixels were written; None for an unbaked surface
    pub tile: Option<(RegionPos, u64)>,
}
