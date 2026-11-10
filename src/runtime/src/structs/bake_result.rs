use minimap_core::structs::pos::RegionPos;

use crate::structs::BakedChunk;

#[derive(Debug)]
pub enum BakeResult {
    Chunk { chunk: Box<BakedChunk>, soft_only: bool, tile_version: u64 },
    Region { region: RegionPos, chunks: Vec<BakedChunk>, tile_version: u64 },
    // the chunk left memory, or a full job will cover it
    Skipped,
}
