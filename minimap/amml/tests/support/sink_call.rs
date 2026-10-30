use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::structs::SubmitStatus;
use minimap_store::structs::WorldIdentity;

// one call that reached a RecordingSink
#[derive(Debug, Clone, PartialEq)]
pub enum SinkCall {
    Enter(WorldIdentity, i32),
    Leave,
    Tick { present: bool },
    Submit(ChunkPos, SubmitStatus),
    BlockChanged(i32, i32, i32),
    Column(ChunkPos, usize),
    BiomeName(u16, String),
}
