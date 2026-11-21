use minimap_core::structs::chunk::ColumnSample;
use minimap_core::structs::pos::ChunkPos;
use minimap_store::structs::WorldIdentity;

#[derive(Debug, Clone, PartialEq)]
pub enum SinkCall {
    Enter(WorldIdentity, i32),
    Leave,
    BlockChanged(i32, i32, i32),
    Column(ChunkPos, usize, ColumnSample),
    BiomeName(u16, String),
}
