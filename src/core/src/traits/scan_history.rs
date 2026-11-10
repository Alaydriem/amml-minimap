use crate::structs::pos::ChunkPos;

pub trait ScanHistory {
    // tick of the last scan that completed without a placeholder
    fn last_scan(&self, pos: ChunkPos) -> Option<u64>;
}
